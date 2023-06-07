# Download assets.
curl -s --stderr - \
https://api.github.com/repos/nanoskript/danbooru-tags/releases/tags/assets \
| grep "browser_download_url.*sqlite.gz" \
| cut -d : -f 2,3 \
| tr -d \" \
| xargs curl -L --remote-name-all

# Decompress and delete original.
for f in ./*.sqlite.gz; do
  gzip -df "$f"
done
