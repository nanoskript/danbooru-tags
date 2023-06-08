import itertools
from pathlib import Path

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from starlette.responses import RedirectResponse
from marisa_trie import Trie
from sqlitedict import SqliteDict

app = FastAPI(title="danbooru-tags-explorer")
app.add_middleware(CORSMiddleware, allow_origins=["*"])


def load_table(table):
    return SqliteDict(Path("vendor") / table, flag="r")


tags = load_table("tags.sqlite")
tag_names_to_ids = load_table("tag_names_to_ids.sqlite")
tags_with_rankings = load_table("tags_with_rankings.sqlite")
tags_with_posts_over_time = load_table("tags_with_posts_over_time.sqlite")
tag_names_trie = Trie(tag_names_to_ids.keys())


class TagCompletion(BaseModel):
    tag: str
    category: int


class TagCorrelation(BaseModel):
    tag: str
    tag_category: int
    n_correlated: int


class TagCorrelations(BaseModel):
    n_posts_for_tag: int
    correlations: list[TagCorrelation]


@app.get("/", include_in_schema=False)
async def route_index():
    return RedirectResponse("/docs")


@app.get("/tag_complete")
async def route_tag_complete(prefix: str) -> list[TagCompletion]:
    all_tag_names = tag_names_trie.iterkeys(prefix)
    tag_names = list(itertools.islice(all_tag_names, 10))

    completions = []
    for name in tag_names:
        tag_id = tag_names_to_ids[name]
        completions.append(TagCompletion(
            tag=name,
            category=tags[str(tag_id)]["category"],
        ))
    return completions


@app.get("/tag_posts_over_time")
async def route_tag_posts_over_time(tag: str) -> list[tuple[str, int]]:
    try:
        tag_id = tag_names_to_ids[tag]
        return tags_with_posts_over_time[str(tag_id)]
    except KeyError:
        raise HTTPException(status_code=404)


@app.get("/tag_correlations")
async def route_tag_correlations(tag: str) -> TagCorrelations:
    try:
        tag_id = tag_names_to_ids[tag]
        n_posts_for_tag = tags[str(tag_id)]["post_count"]
        rankings = tags_with_rankings[str(tag_id)]
    except KeyError:
        raise HTTPException(status_code=404)

    return TagCorrelations(
        n_posts_for_tag=n_posts_for_tag,
        correlations=[
            TagCorrelation(
                tag=tags[str(tag_id)]["name"],
                tag_category=tags[str(tag_id)]["category"],
                n_correlated=n_correlated,
            )
            for tag_id, n_correlated
            in rankings
        ]
    )
