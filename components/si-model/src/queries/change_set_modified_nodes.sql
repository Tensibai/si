SELECT entities_change_set_projection.id
FROM entities_change_set_projection
LEFT OUTER JOIN entities_head ON entities_change_set_projection.id = entities_head.id
WHERE entities_change_set_projection.change_set_id = si_id_to_primary_key_v1($1)
  AND entities_head.id IS NOT NULL
  AND entities_change_set_projection.obj -> 'siStorable' -> 'deleted' = 'false'::jsonb;