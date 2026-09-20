# In-tree build of the current checkout. From the repository root:
#   makepkg -si
#
# Maintainer: Josef Vybihal <josef.vybihal@gmail.com>
pkgname=herdr-git
_pkgname=${pkgname%-git}
_abbr=7

# Cargo.toml version + commits-since-tag + short hash, e.g. 0.9.1.r168.g6f7818d.
# Not named pkgver() so makepkg will not rewrite this file.
_herdr_root() {
  if [[ -n ${startdir:-} ]]; then
    printf '%s\n' "$startdir"
  else
    cd "$(dirname "${BASH_SOURCE[0]}")" && pwd
  fi
}

_herdr_pkgver() {
  local root cargo_ver desc commits hash
  root=$(_herdr_root)
  cargo_ver=$(awk -F'"' '/^version = / { print $2; exit }' "$root/Cargo.toml")
  cargo_ver=${cargo_ver//-/.}
  if desc=$(git -C "$root" describe --long --tags --abbrev="$_abbr" --match='v*' HEAD 2>/dev/null); then
    desc=${desc#v}
    desc=$(sed 's/\([^-]*-g\)/r\1/;s/-/./g' <<<"$desc")
    printf '%s.%s\n' "$cargo_ver" "${desc#*.*.*.}"
  else
    commits=$(git -C "$root" rev-list --count HEAD)
    hash=$(git -C "$root" rev-parse --short="$_abbr" HEAD)
    printf '%s.r%s.g%s\n' "$cargo_ver" "$commits" "$hash"
  fi
}

pkgver=$(_herdr_pkgver)
pkgrel=1
pkgdesc='Terminal workspace manager for AI coding agents'
arch=('x86_64' 'aarch64')
url='https://herdr.dev'
_url="https://github.com/herdrdev/$_pkgname"
license=('Apache-2.0')
depends=('gcc-libs' 'glibc' 'libgcc')
makedepends=('cargo' 'cmake' 'git' 'zig')
provides=("$_pkgname=${pkgver%.*.*}")
conflicts=("$_pkgname")
options=('strip' '!staticlibs' '!zipman' '!debug' 'buildflags' 'lto')
source=()
sha256sums=()

# Keep makepkg's $srcdir/$pkgdir off this repo's Rust src/.
if [[ ${BUILDDIR:-} -ef ${startdir:-} ]]; then
  BUILDDIR="$startdir/.makepkg"
fi

_srcenv() {
  if [[ -f "$startdir/src/main.rs" && ${srcdir:-} -ef "$startdir/src" ]]; then
    echo 'srcdir is the Rust source tree; refuse to use it as CARGO_HOME' >&2
    return 1
  fi
  cd "$startdir"
  export CARGO_HOME="$srcdir"
  export CARGO_PROFILE_RELEASE_DEBUG=2
  export CARGO_PROFILE_RELEASE_STRIP=false
  export CARGO_PROFILE_RELEASE_LTO=thin
  export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
  export CARGO_PROFILE_RELEASE_OPT_LEVEL=3
  CFLAGS+=' -ffat-lto-objects'
  CXXFLAGS+=' -ffat-lto-objects'
  RUSTFLAGS+=" --remap-path-prefix $PWD=/"
  export LIBSQLITE3_SYS_USE_PKG_CONFIG=1
  export ZSTD_SYS_USE_PKG_CONFIG=1
  export LIBGHOSTTY_VT_OPTIMIZE=ReleaseFast
  export LIBGHOSTTY_VT_SIMD=true
  export ZIG_GLOBAL_CACHE_DIR="$srcdir/zig-cache/"
}

prepare() {
  _srcenv
  cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
  cd vendor/libghostty-vt && zig build --fetch=all
}

build() {
  _srcenv
  source /etc/os-release || true
  export CARGO_TARGET_DIR=target
  export HERDR_BUILD_COMMIT="${pkgver: -"$_abbr"}"
  export HERDR_BUILD_CHANNEL="${pkgname#$_pkgname-}"
  export HERDR_BUILD_ID="${pkgver#*.*.*.}${ID:+.$ID}"
  cargo build --release --frozen --config "package.version='$pkgver'"
  target/release/$_pkgname completion bash >"$srcdir/completions.bash"
  target/release/$_pkgname completion fish >"$srcdir/completions.fish"
  target/release/$_pkgname completion zsh >"$srcdir/completions.zsh"
}

package() {
  cd "$startdir"
  install -Dm755 target/release/$_pkgname "$pkgdir/usr/bin/$_pkgname"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$_pkgname/LICENSE"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$_pkgname/README.md"
  install -Dm0644 "$srcdir/completions.bash" "$pkgdir/usr/share/bash-completion/completions/$_pkgname"
  install -Dm0644 "$srcdir/completions.fish" "$pkgdir/usr/share/fish/vendor_completions.d/$_pkgname.fish"
  install -Dm0644 "$srcdir/completions.zsh" "$pkgdir/usr/share/zsh/site-functions/_$_pkgname"
}
