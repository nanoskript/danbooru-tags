FROM python:3.10-slim-buster AS base

RUN pip install --no-cache-dir pdm
ADD ./pyproject.toml ./pdm.lock ./
RUN pdm sync && pdm cache clear

FROM base AS builder

ADD ./vendor ./vendor
ADD ./json_to_sqlite.py ./

RUN mkdir data
RUN pdm run json_to_sqlite.py vendor/tags.json.gz data/tags.sqlite
RUN pdm run json_to_sqlite.py vendor/tag_names_to_ids.json.gz data/tag_names_to_ids.sqlite
RUN pdm run json_to_sqlite.py vendor/tags_with_rankings.json.gz data/tags_with_rankings.sqlite

FROM base

COPY --from=builder ./data ./data
ADD ./main.py ./

CMD ["pdm", "run", "uvicorn", \
	"--host", "0.0.0.0", "--port", "$PORT", \
	"main:app"]
