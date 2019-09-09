pkg_name=si-graphql
pkg_origin=si
pkg_version="0.0.1"
pkg_maintainer="System Initiative <adam@systeminit.com>"
pkg_license=("Proprietary")
pkg_shasum="TODO"
pkg_deps=(core/node core/sqlite)
pkg_build_deps=(core/gcc core/python2 core/make core/coreutils)

do_build() {
  ln -sv $(pkg_path_for core/coreutils)/bin/env /usr/bin/env
  npm install --build-from-source --sqlite=$(pkg_path_for core/sqlite) --loglevel verbose
  npm run build
  npm prune  --production
}

do_install() {
  cp -r ./node_modules $pkg_prefix
  cp -r ./dist $pkg_prefix
}

do_strip() {
  return 0;
}
