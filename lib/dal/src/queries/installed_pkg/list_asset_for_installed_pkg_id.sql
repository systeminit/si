SELECT row_to_json(ipa.*) AS object
FROM installed_pkg_assets_v1($1, $2) as ipa 
WHERE 
  ipa.installed_pkg_id = $3
