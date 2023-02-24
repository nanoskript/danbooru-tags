mkdir -p ./vendor
cp ../../processed/tags.json ./vendor/.
cp ../../processed/tag_names_to_ids.json ./vendor/.
cp ../../processed/tags_with_rankings.json ./vendor/.
gzip -v ./vendor/tags.json
gzip -v ./vendor/tag_names_to_ids.json
gzip -v ./vendor/tags_with_rankings.json
