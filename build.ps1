if ($args.Count -eq 0) {
    $BUILD_BOTH = $true
}

$EMBED_FRONTEND = $false
$BUILD_LITE = $false
$BUILD_FULL = $false
$TARGET = ""

for ($i = 0; $i -lt $args.Count; $i++) {
    switch ($args[$i]) {
        "--embed-frontend" {
            $EMBED_FRONTEND = $true
        }
        "--lite" {
            $BUILD_LITE = $true
        }
        "--full" {
            $BUILD_FULL = $true
        }
        "--both" {
            $BUILD_BOTH = $true
        }
        "--target" {
            $TARGET = $args[$i + 1]
            $i++
        }
    }
}

if (-not $BUILD_LITE -and -not $BUILD_FULL) {
    $BUILD_BOTH = $true
}

if ([string]::IsNullOrEmpty($TARGET)) {
    $TARGET = "x86_64-pc-windows-msvc"
    Write-Host "Using target: $TARGET"
}

Write-Host "Building memos-rs..."

Write-Host "Building frontend..."
cd ./frontend
npm install
npm run build
cd ..

if ($BUILD_LITE -or $BUILD_BOTH) {
    Write-Host "Building Lite version..."
    if ($EMBED_FRONTEND) {
        Write-Host "Using embedded frontend for lite version"
        cargo build --release --no-default-features --features "lite embed-frontend" --bin memos-rs-lite --target "$TARGET"
    } else {
        cargo build --release --no-default-features --features "lite" --bin memos-rs-lite --target "$TARGET"
    }
    Write-Host "Lite version build complete!"
}

if ($BUILD_FULL -or $BUILD_BOTH) {
    Write-Host "Building Full version..."
    if ($EMBED_FRONTEND) {
        Write-Host "Embedding frontend into binary..."
        cargo build --release --features "embed-frontend" --target "$TARGET"
    } else {
        cargo build --release --target "$TARGET"
    }
    Write-Host "Full version build complete!"
}

Write-Host "Build complete!"
