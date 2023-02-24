mkdir ./data
cp ../../processed/tags.json ./data/.
cp ../../processed/tag_names_to_ids.json ./data/.
cp ../../processed/tags_with_rankings.json ./data/.
gzip -v ./data/tags.json
gzip -v ./data/tag_names_to_ids.json
gzip -v ./data/tags_with_rankings.json
