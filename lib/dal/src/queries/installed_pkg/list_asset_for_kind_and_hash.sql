SELECT row_to_json(ipa.*) AS object
FROM installed_pkg_assets_v1($1, $2) as ipa -- norcal style
WHERE 
  ipa.asset_kind = $3 
  AND ipa.asset_hash = $4
