$headers = @{ 'User-Agent' = 'cocoBar-Updater'; Accept = 'application/vnd.github+json' }
$release = Invoke-RestMethod -Uri 'https://api.github.com/repos/phon-t/CocoBar/releases/latest' -Headers $headers -UseBasicParsing -TimeoutSec 20
if ($release.draft -or $release.prerelease) { throw 'GitHub returned a draft or prerelease.' }
$asset = $release.assets | Where-Object { $_.name -ceq 'cocobar.exe' -and $_.state -eq 'uploaded' } | Select-Object -First 1
if ($null -eq $asset) { '{0}|||' -f $release.tag_name }
else { '{0}|{1}|{2}|{3}' -f $release.tag_name, $asset.browser_download_url, $asset.size, $asset.digest }
