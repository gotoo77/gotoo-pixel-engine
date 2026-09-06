$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$Example = "gpe_ui_p4_font_gallery_web"
$Wasm = Join-Path $Root "target/wasm32-unknown-unknown/debug/examples/$Example.wasm"
$OutDir = Join-Path $Root "web/pkg"

Push-Location $Root
try {
    Write-Host "==> cargo build --target wasm32-unknown-unknown --features outline-fonts --example $Example"
    cargo build --target wasm32-unknown-unknown --features outline-fonts --example $Example
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }

    New-Item -ItemType Directory -Force $OutDir | Out-Null

    Write-Host "==> wasm-bindgen --target web --out-dir web/pkg $Wasm"
    wasm-bindgen --target web --out-dir $OutDir $Wasm
    if ($LASTEXITCODE -ne 0) {
        throw "wasm-bindgen failed with exit code $LASTEXITCODE"
    }

    Write-Host "==> P4 Web font gallery ready"
    Write-Host "==> Serve with: python .\scripts\dev.py serve-web"
    Write-Host "==> Open: http://127.0.0.1:8000/gpe_ui_p4_font_gallery.html"
}
finally {
    Pop-Location
}
