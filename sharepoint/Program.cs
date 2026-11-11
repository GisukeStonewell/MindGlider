using System.Net;
using System.Net.Http.Headers;
using System.Text.Json;
using System.Text.RegularExpressions;
using Microsoft.Identity.Client;
using Microsoft.Identity.Client.Broker;

if (args is ["--self-test"])
    return await GraphTests.Run();
if (args.Length != 5 || !Guid.TryParse(args[0], out var tenant) ||
    !Guid.TryParse(args[1], out var client) || !long.TryParse(args[4], out var window) ||
    window == 0 || !IsItemId(args[2]) || !IsItemId(args[3]))
    return 4;

try
{
    var request = JsonSerializer.Deserialize<SyncRequest>(await Console.In.ReadLineAsync() ?? "")
        ?? throw new JsonException();
    using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(request.Interactive ? 300 : 30));
    var token = deadline.Token;
    var app = PublicClientApplicationBuilder.Create(client.ToString())
        .WithAuthority(AzureCloudInstance.AzurePublic, tenant.ToString())
        .WithRedirectUri($"ms-appx-web://microsoft.aad.brokerplugin/{client}")
        .WithParentActivityOrWindow(() => new IntPtr(window))
        .WithBroker(new BrokerOptions(BrokerOptions.OperatingSystems.Windows) { Title = "MindGlider" })
        .Build();
    string[] scopes = ["https://graph.microsoft.com/Files.ReadWrite"];
    var account = request.Account is null
        ? PublicClientApplication.OperatingSystemAccount
        : await app.GetAccountAsync(request.Account);
    AuthenticationResult identity;
    try
    {
        if (account is null)
            throw new MsalUiRequiredException("account_selection", "Account selection required.");
        identity = await app.AcquireTokenSilent(scopes, account).ExecuteAsync(token);
    }
    catch (MsalUiRequiredException) when (request.Interactive)
    {
        var interactive = app.AcquireTokenInteractive(scopes);
        if (account is not null && account != PublicClientApplication.OperatingSystemAccount)
            interactive = interactive.WithAccount(account);
        identity = await interactive.ExecuteAsync(token);
    }
    if (!Guid.TryParse(identity.TenantId, out var actualTenant) || actualTenant != tenant ||
        identity.Account.HomeAccountId.Identifier is not { Length: > 0 } accountId ||
        (request.Account is not null && request.Account != accountId))
        return 2;

    using var http = new HttpClient(new HttpClientHandler { AllowAutoRedirect = false });
    var graph = new GraphSync(http, args[2], args[3], identity.AccessToken, accountId);
    var result = request.Bytes is null
        ? await graph.Read(request.Tag, token)
        : await graph.Save(request, token);
    Console.WriteLine(JsonSerializer.Serialize(result));
    return 0;
}
catch (GraphFailure error)
{
    return error.ExitCode;
}
catch (MsalException)
{
    return 2;
}
catch (Exception)
{
    // Broker exceptions and preauthenticated transfer URLs can contain credentials.
    return 5;
}

static bool IsItemId(string value) => Regex.IsMatch(value, @"\A[A-Za-z0-9!_-]{1,512}\z");

record SyncRequest(string? Account, string? Tag, byte[]? Bytes, bool Interactive);
record SyncResponse(string Account, string? Tag, byte[]? Bytes);
sealed class GraphFailure(int exitCode) : Exception
{
    public int ExitCode { get; } = exitCode;
}
