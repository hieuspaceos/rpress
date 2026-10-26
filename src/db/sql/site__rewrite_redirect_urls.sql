UPDATE redirects SET new_url = REPLACE(new_url, ?1, ?2) WHERE new_url LIKE '%' || ?1 || '%';
