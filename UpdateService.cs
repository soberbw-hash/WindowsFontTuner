using System;
using System.IO;
using System.Net;
using System.Text.RegularExpressions;
using System.Web.Script.Serialization;

namespace WindowsFontTuner
{
    public sealed class UpdateService
    {
        private const string LatestReleaseApi = "https://api.github.com/repos/soberbw-hash/WindowsFontTuner/releases/latest";
        public const string RepositoryPage = "https://github.com/soberbw-hash/WindowsFontTuner";
        public const string LatestReleasePage = "https://github.com/soberbw-hash/WindowsFontTuner/releases/latest";

        public UpdateCheckResult CheckForUpdates(Version currentVersion)
        {
            Version normalizedCurrent = NormalizeVersion(currentVersion);
            ServicePointManager.SecurityProtocol = SecurityProtocolType.Tls12;

            HttpWebRequest request = (HttpWebRequest)WebRequest.Create(LatestReleaseApi);
            request.Method = "GET";
            request.Accept = "application/vnd.github+json";
            request.UserAgent = "WindowsFontTuner";
            request.Timeout = 8000;
            request.ReadWriteTimeout = 8000;
            request.AutomaticDecompression = DecompressionMethods.GZip | DecompressionMethods.Deflate;
            request.Headers["X-GitHub-Api-Version"] = "2022-11-28";

            using (HttpWebResponse response = (HttpWebResponse)request.GetResponse())
            using (Stream stream = response.GetResponseStream())
            using (StreamReader reader = new StreamReader(stream))
            {
                string json = reader.ReadToEnd();
                JavaScriptSerializer serializer = new JavaScriptSerializer();
                GitHubReleaseResponse release = serializer.Deserialize<GitHubReleaseResponse>(json);

                Version latestVersion = ParseVersion(release == null ? null : release.tag_name);
                Version normalizedLatest = NormalizeVersion(latestVersion);

                UpdateCheckResult result = new UpdateCheckResult();
                result.CurrentVersion = normalizedCurrent;
                result.LatestVersion = normalizedLatest;
                result.LatestTag = release == null ? null : release.tag_name;
                result.ReleaseName = release == null ? null : release.name;
                result.ReleaseUrl = string.IsNullOrWhiteSpace(release == null ? null : release.html_url)
                    ? LatestReleasePage
                    : release.html_url;
                result.ReleaseBody = release == null ? null : release.body;
                result.DownloadUrl = GetBestAssetUrl(release);
                result.PublishedAt = ParseDate(release == null ? null : release.published_at);
                result.IsUpdateAvailable = normalizedLatest != null && normalizedLatest > normalizedCurrent;
                return result;
            }
        }

        private static string GetBestAssetUrl(GitHubReleaseResponse release)
        {
            if (release == null || release.assets == null || release.assets.Length == 0)
            {
                return LatestReleasePage;
            }

            foreach (GitHubReleaseAssetResponse asset in release.assets)
            {
                if (asset != null && !string.IsNullOrWhiteSpace(asset.browser_download_url) &&
                    asset.name != null && asset.name.EndsWith(".zip", StringComparison.OrdinalIgnoreCase))
                {
                    return asset.browser_download_url;
                }
            }

            foreach (GitHubReleaseAssetResponse asset in release.assets)
            {
                if (asset != null && !string.IsNullOrWhiteSpace(asset.browser_download_url))
                {
                    return asset.browser_download_url;
                }
            }

            return string.IsNullOrWhiteSpace(release.html_url) ? LatestReleasePage : release.html_url;
        }

        private static DateTime? ParseDate(string text)
        {
            DateTime value;
            return DateTime.TryParse(text, out value) ? value : (DateTime?)null;
        }

        private static Version ParseVersion(string tag)
        {
            if (string.IsNullOrWhiteSpace(tag))
            {
                return null;
            }

            Match match = Regex.Match(tag, @"(\d+)(\.\d+){0,3}");
            if (!match.Success)
            {
                return null;
            }

            string[] parts = match.Value.Split('.');
            int major = parts.Length > 0 ? ParsePart(parts[0]) : 0;
            int minor = parts.Length > 1 ? ParsePart(parts[1]) : 0;
            int build = parts.Length > 2 ? ParsePart(parts[2]) : 0;
            int revision = parts.Length > 3 ? ParsePart(parts[3]) : 0;
            return new Version(major, minor, build, revision);
        }

        private static int ParsePart(string text)
        {
            int value;
            return int.TryParse(text, out value) ? value : 0;
        }

        private static Version NormalizeVersion(Version version)
        {
            if (version == null)
            {
                return null;
            }

            return new Version(
                Math.Max(version.Major, 0),
                Math.Max(version.Minor, 0),
                Math.Max(version.Build, 0),
                Math.Max(version.Revision, 0));
        }

        private sealed class GitHubReleaseResponse
        {
            public string tag_name { get; set; }
            public string name { get; set; }
            public string body { get; set; }
            public string html_url { get; set; }
            public string published_at { get; set; }
            public GitHubReleaseAssetResponse[] assets { get; set; }
        }

        private sealed class GitHubReleaseAssetResponse
        {
            public string name { get; set; }
            public string browser_download_url { get; set; }
        }
    }

    public sealed class UpdateCheckResult
    {
        public Version CurrentVersion { get; set; }
        public Version LatestVersion { get; set; }
        public string LatestTag { get; set; }
        public string ReleaseName { get; set; }
        public string ReleaseUrl { get; set; }
        public string DownloadUrl { get; set; }
        public string ReleaseBody { get; set; }
        public DateTime? PublishedAt { get; set; }
        public bool IsUpdateAvailable { get; set; }
    }
}
