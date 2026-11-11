using System.Net;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Text.Json;

sealed class GraphSync(HttpClient http, string driveId, string itemId, string accessToken, string account)
{
    public const int MaxBytes = 16 * 1024 * 1024;
    const int PartSize = 320 * 1024 * 16;
    readonly string item = $"https://graph.microsoft.com/v1.0/drives/{driveId}/items/{itemId}";
    record Metadata(string Tag, string Name, string Parent);

    public async Task<SyncResponse> Read(string? tag, CancellationToken token)
    {
        var metadata = await GetMetadata(tag, token);
        if (metadata is null)
            return new(account, null, null);

        using var response = await Send(HttpMethod.Get, item + "/content", token);
        byte[] bytes;
        if (response.StatusCode == HttpStatusCode.Found)
        {
            var url = TransferUrl(response.Headers.Location);
            using var download = await http.GetAsync(url, HttpCompletionOption.ResponseHeadersRead, token);
            bytes = await ReadBytes(download, token);
        }
        else
        {
            bytes = await ReadBytes(response, token);
        }

        // Bind the downloaded bytes to an exact revision, including a write racing the redirect.
        if (await GetMetadata(metadata.Tag, token) is not null)
            throw new GraphFailure(6);
        return new(account, metadata.Tag, bytes);
    }

    public async Task<SyncResponse> Save(SyncRequest request, CancellationToken token)
    {
        if (request.Tag is not { Length: > 0 } tag || request.Bytes is not { Length: > 0 and <= MaxBytes } bytes)
            throw new GraphFailure(4);

        var metadata = await GetMetadata(null, token) ?? throw new GraphFailure(5);
        if (metadata.Tag != tag)
            throw new GraphFailure(6);

        using var session = await Send(HttpMethod.Post, item + "/createUploadSession", token,
            JsonContent.Create(new
            {
                deferCommit = true,
                item = new Dictionary<string, string> { ["@microsoft.graph.conflictBehavior"] = "replace" },
            }), "If-Match", tag);
        Check(session);
        using var json = JsonDocument.Parse(await session.Content.ReadAsStreamAsync(token));
        var uploadUrl = TransferUrl(new Uri(json.RootElement.GetProperty("uploadUrl").GetString()!));
        try
        {
            for (int offset = 0; offset < bytes.Length; offset += PartSize)
            {
                int length = Math.Min(PartSize, bytes.Length - offset);
                using var part = new ByteArrayContent(bytes, offset, length);
                part.Headers.ContentRange = new ContentRangeHeaderValue(offset, offset + length - 1, bytes.Length);
                part.Headers.ContentType = new MediaTypeHeaderValue("application/octet-stream");
                using var upload = await http.PutAsync(uploadUrl, part, token);
                Check(upload);
                // Deferred uploads must remain staged until the conditional commit.
                if (upload.StatusCode != HttpStatusCode.Accepted)
                    throw new GraphFailure(5);
            }
            var target = $"https://graph.microsoft.com/v1.0/drives/{driveId}/items/{Uri.EscapeDataString(metadata.Parent)}:/{Uri.EscapeDataString(metadata.Name)}:";
            using var commit = await Send(HttpMethod.Put, target, token, JsonContent.Create(new Dictionary<string, string>
            {
                ["name"] = metadata.Name,
                ["@microsoft.graph.sourceUrl"] = uploadUrl.AbsoluteUri,
                ["@microsoft.graph.conflictBehavior"] = "replace",
            }), "If-Match", tag);
            Check(commit);
            using var saved = JsonDocument.Parse(await commit.Content.ReadAsStreamAsync(token));
            if (commit.StatusCode != HttpStatusCode.OK || saved.RootElement.GetProperty("id").GetString() != itemId)
                throw new GraphFailure(5);
            return new(account, RequiredTag(saved.RootElement), null);
        }
        catch
        {
            using var cleanup = new CancellationTokenSource(TimeSpan.FromSeconds(3));
            try { using var cancelled = await http.DeleteAsync(uploadUrl, cleanup.Token); }
            catch (HttpRequestException) { }
            catch (OperationCanceledException) { }
            throw;
        }
    }

    async Task<Metadata?> GetMetadata(string? tag, CancellationToken token)
    {
        using var response = await Send(HttpMethod.Get, item + "?$select=id,name,file,size,eTag,parentReference",
            token, header: "If-None-Match", tag: tag);
        if (response.StatusCode == HttpStatusCode.NotModified && tag is not null)
            return null;
        Check(response);
        using var properties = JsonDocument.Parse(await response.Content.ReadAsStreamAsync(token));
        var map = properties.RootElement;
        var name = map.GetProperty("name").GetString() ?? "";
        if (map.GetProperty("id").GetString() != itemId || !map.TryGetProperty("file", out _) ||
            !name.EndsWith(".mm", StringComparison.OrdinalIgnoreCase) ||
            name.IndexOfAny(['/', '\\']) >= 0 || map.GetProperty("size").GetInt64() > MaxBytes)
            throw new GraphFailure(4);
        var parent = map.GetProperty("parentReference").GetProperty("id").GetString();
        if (string.IsNullOrWhiteSpace(parent))
            throw new GraphFailure(4);
        return new(RequiredTag(map), name, parent);
    }

    async Task<HttpResponseMessage> Send(HttpMethod method, string url, CancellationToken token,
        HttpContent? content = null, string? header = null, string? tag = null)
    {
        using var request = new HttpRequestMessage(method, url) { Content = content };
        request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", accessToken);
        if (header is not null && tag is not null && !request.Headers.TryAddWithoutValidation(header, tag))
            throw new GraphFailure(4);
        return await http.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, token);
    }

    static string RequiredTag(JsonElement item)
    {
        var tag = item.GetProperty("eTag").GetString();
        if (!EntityTagHeaderValue.TryParse(tag, out var entity) || entity.IsWeak ||
            entity == EntityTagHeaderValue.Any)
            throw new GraphFailure(4);
        return entity.ToString();
    }

    static Uri TransferUrl(Uri? url)
    {
        if (url is not { IsAbsoluteUri: true, Scheme: "https", IsDefaultPort: true } ||
            url.UserInfo.Length != 0 || url.Fragment.Length != 0)
            throw new GraphFailure(5);
        return url;
    }

    static void Check(HttpResponseMessage response)
    {
        if (!response.IsSuccessStatusCode)
            throw new GraphFailure(response.StatusCode switch
            {
                HttpStatusCode.Unauthorized or HttpStatusCode.Forbidden or HttpStatusCode.NotFound => 3,
                HttpStatusCode.Conflict or HttpStatusCode.PreconditionFailed => 6,
                (HttpStatusCode)423 => 7,
                _ => 5,
            });
    }

    static async Task<byte[]> ReadBytes(HttpResponseMessage response, CancellationToken token)
    {
        Check(response);
        await using var content = await response.Content.ReadAsStreamAsync(token);
        using var output = new MemoryStream();
        var buffer = new byte[65536];
        int count;
        while ((count = await content.ReadAsync(buffer, token)) != 0)
        {
            if (output.Length + count > MaxBytes)
                throw new GraphFailure(4);
            output.Write(buffer, 0, count);
        }
        return output.ToArray();
    }
}
