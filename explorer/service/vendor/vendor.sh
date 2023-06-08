cp ../../../processed/tags.json ./
cp ../../../processed/tag_names_to_ids.json ./
cp ../../../processed/tags_with_rankings.json ./
cp ../../../processed/tags_with_posts_over_time.json ./

for f in ./*.json; do
  pdm run json_to_sqlite.py "$f" "${f%.json}.sqlite"
  gzip -vf --keep "${f%.json}.sqlite"
  rm "$f"
done
