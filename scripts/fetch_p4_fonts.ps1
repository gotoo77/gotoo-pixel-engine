$ErrorActionPreference = 'Stop'
$families = @('abel','abeezee','abrilfatface','acme','alata','aldrich','alice','amaranth','amaticsc','architectsdaughter','arvo','bangers','belleza','bitter','cabin','caveat','cinzel','comfortaa','cormorantgaramond','dancingscript')
$families += @('ebgaramond','exo2','figtree','fraunces','greatvibes','heebo','hind','inconsolata','indieflower','josefinsans','jost','karla','lato','lora','merriweather','montserrat','nunito','oswald','outfit','pacifico','playfairdisplay','quicksand','raleway','rubik','sacramento','teko','unbounded','vollkorn','worksans','xanhmono','yanonekaffeesatz','zillaslab')
$headers = @{ 'User-Agent' = 'GPE-P4-font-gallery' }
$root = Join-Path $PSScriptRoot '../assets/fonts/p4'
$old = $null
if (Test-Path (Join-Path $root 'sources.json')) { $old = Get-Content (Join-Path $root 'sources.json') -Raw | ConvertFrom-Json }
$revision = if ($old) { $old.revision } else { (Invoke-RestMethod 'https://api.github.com/repos/google/fonts/commits/main' -Headers $headers).sha }
New-Item -ItemType Directory -Force -Path $root | Out-Null
$manifest = foreach ($family in $families) {
    $existing = @($old.fonts) | Where-Object { $_.family -eq $family } | Select-Object -First 1
    if ($existing -and (Test-Path (Join-Path $root "$family/font.ttf")) -and (Test-Path (Join-Path $root "$family/OFL.txt"))) { $existing; continue }
    $files = Invoke-RestMethod "https://api.github.com/repos/google/fonts/contents/ofl/${family}?ref=$revision" -Headers $headers
    $font = $files | Where-Object { $_.name -like '*.ttf' } | Sort-Object @{ Expression = { if ($_.name -like '*Regular*') { 0 } elseif ($_.name -like '*Italic*') { 2 } else { 1 } } }, name | Select-Object -First 1
    if (-not $font) { throw "No TTF for $family" }
    $dest = Join-Path $root $family
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    Invoke-WebRequest $font.download_url -OutFile (Join-Path $dest 'font.ttf')
    Invoke-WebRequest "https://raw.githubusercontent.com/google/fonts/$revision/ofl/$family/OFL.txt" -OutFile (Join-Path $dest 'OFL.txt')
    Write-Host $family
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $digest = [BitConverter]::ToString($sha.ComputeHash([IO.File]::ReadAllBytes((Join-Path $dest 'font.ttf')))).Replace('-', '')
    $sha.Dispose()
    [pscustomobject]@{ family = $family; source = $font.download_url; sha256 = $digest }
}
[pscustomobject]@{ revision = $revision; fonts = @($manifest) } | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $root 'sources.json')
