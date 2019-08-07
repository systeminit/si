pkg_name=si-graphql
pkg_origin=si
pkg_version="0.0.1"
pkg_maintainer="System Initiative <adam@systeminit.com>"
pkg_license=("Proprietary")
pkg_shasum="TODO"
pkg_deps=(core/node core/sqlite)
pkg_build_deps=(core/gcc core/python2 core/make)

do_build() {
  npm install
  npm run build
  npm prune  --production
}

do_install() {
  cp -r ./node_modules $pkg_prefix
  cp -r knexfile.js $pkg_prefix
  cp -r migrations $pkg_prefix
  cp -r ./dist $pkg_prefix
}

do_strip() {
  return 0;
}
