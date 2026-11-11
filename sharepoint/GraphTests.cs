using System.Net;
using System.Text;
using System.Text.Json;

static class GraphTests
{
    public static async Task<int> Run()
    {
        using var mock = new SharePoint();
        using var http = new HttpClient(mock);
        var first = new GraphSync(http, "drive", "item", "mock", "first");
        var second = new GraphSync(http, "drive", "item", "mock", "second");
        var a = await first.Read(null, default);
        var b = await second.Read(null, default);
        Require(a.Tag == b.Tag && a.Bytes!.SequenceEqual(new byte[] { 1 }));
        Require((await first.Read(a.Tag, default)).Bytes is null);
        var saved = await first.Save(new("first", a.Tag, [2], false), default);
        Require(saved.Tag != a.Tag && mock.Current.SequenceEqual(new byte[] { 2 }));
        await Conflict(() => second.Save(new("second", b.Tag, [3], false), default));
        var updated = await second.Read(b.Tag, default);
        Require(updated.Bytes!.SequenceEqual(new byte[] { 2 }));

        mock.RaceAfterUpload = true;
        await Conflict(() => second.Save(new("second", updated.Tag, [3], false), default));
        Require(mock.Current.SequenceEqual(new byte[] { 9 }) && mock.Cancelled);
        mock.RaceOnDownload = true;
        await Conflict(() => first.Read(null, default));
        Console.WriteLine("Virtual SharePoint: load, poll, save, stale write, commit race, read race passed.");
        return 0;
    }

    static async Task Conflict(Func<Task<SyncResponse>> action)
    {
        try { await action(); }
        catch (GraphFailure error) when (error.ExitCode == 6) { return; }
        throw new InvalidOperationException("Expected stale-write rejection.");
    }

    static void Require(bool value)
    {
        if (!value)
            throw new InvalidOperationException("Unexpected Graph sync behavior.");
    }

    sealed class SharePoint : HttpMessageHandler
    {
        public byte[] Current { get; private set; } = [1];
        public bool RaceAfterUpload { get; set; }
        public bool RaceOnDownload { get; set; }
        public bool Cancelled { get; private set; }
        byte[]? staged;
        int version = 1;
        string Tag => $"\"tag{version}\"";

        protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken token)
        {
            var uri = request.RequestUri!;
            var graph = uri.Host == "graph.microsoft.com";
            Require(graph ? request.Headers.Authorization?.Parameter == "mock" : request.Headers.Authorization is null);
            if (request.Method == HttpMethod.Get && uri.Host == "transfer.example")
            {
                var response = new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(Current) };
                if (RaceOnDownload) version++;
                return response;
            }
            if (request.Method == HttpMethod.Get && uri.AbsolutePath.EndsWith("/content"))
            {
                var response = new HttpResponseMessage(HttpStatusCode.Found);
                response.Headers.Location = new Uri("https://transfer.example/download");
                return response;
            }
            if (request.Method == HttpMethod.Get)
                return request.Headers.IfNoneMatch.Any(tag => tag.ToString() == Tag)
                    ? new(HttpStatusCode.NotModified) : Item();

            if (request.Method == HttpMethod.Delete && !graph)
            {
                Cancelled = true;
                staged = null;
                return new(HttpStatusCode.NoContent);
            }
            if (request.Method == HttpMethod.Put && !graph)
            {
                Require(request.Content!.Headers.ContentRange!.From == 0);
                staged = await request.Content.ReadAsByteArrayAsync(token);
                if (RaceAfterUpload)
                {
                    Current = [9];
                    version++;
                }
                return new(HttpStatusCode.Accepted);
            }
            if (!request.Headers.IfMatch.Any(tag => tag.ToString() == Tag))
                return new(HttpStatusCode.PreconditionFailed);
            using var body = JsonDocument.Parse(await request.Content!.ReadAsStringAsync(token));
            if (request.Method == HttpMethod.Post)
            {
                Require(uri.AbsolutePath.EndsWith("/createUploadSession") &&
                    body.RootElement.GetProperty("deferCommit").GetBoolean());
                return Json(new { uploadUrl = "https://transfer.example/stage" });
            }

            Require(request.Method == HttpMethod.Put && uri.AbsolutePath.EndsWith("/items/parent:/sample.mm:") &&
                body.RootElement.GetProperty("@microsoft.graph.sourceUrl").GetString() == "https://transfer.example/stage");
            Current = staged ?? throw new InvalidOperationException();
            staged = null;
            version++;
            return Item();
        }

        HttpResponseMessage Item() => Json(new
        {
            id = "item",
            name = "sample.mm",
            size = Current.Length,
            file = new { },
            eTag = Tag,
            parentReference = new { id = "parent" }
        });

        static HttpResponseMessage Json<T>(T body) => new(HttpStatusCode.OK)
        {
            Content = new StringContent(JsonSerializer.Serialize(body), Encoding.UTF8, "application/json")
        };
    }
}
