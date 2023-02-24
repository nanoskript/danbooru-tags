use indicatif::ProgressIterator;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use std::cmp::Reverse;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

const DANBOORU_DATASET_ROOT: &str = "../datasets";

#[derive(Deserialize)]
struct RawTag {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: u64,
    name: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    category: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    post_count: u64,
}

#[derive(Deserialize)]
struct RawPost {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    id: Option<u64>,
    tag_string: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    score: i64,
    rating: String,
    created_at: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct TagId(u64);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct PostId(u64);

#[derive(Serialize, Deserialize)]
struct Tag {
    id: TagId,
    name: String,
    category: u64,
    post_count: u64,
}

#[derive(Serialize, Deserialize)]
struct Post {
    id: PostId,
    tags: Vec<TagId>,
    score: i64,
    rating: char,
    created_at: String,
}

type Tags = FxHashMap<TagId, Tag>;
type TagNamesToIds = FxHashMap<String, TagId>;
type Posts = FxHashMap<PostId, Post>;
type TagPosts = FxHashMap<TagId, Vec<PostId>>;

fn read_all_json_lines<R: DeserializeOwned + Send>(
    glob_pattern: &str,
) -> impl ParallelIterator<Item = R> {
    println!("[read/json/lines] {glob_pattern}");
    let paths: Vec<_> = glob::glob(glob_pattern)
        .unwrap()
        .map(Result::unwrap)
        .collect();

    paths
        .into_iter()
        .progress()
        .flat_map(|path| BufReader::new(File::open(path).unwrap()).lines())
        .map(Result::unwrap)
        .par_bridge()
        .map(|line| serde_json::from_str(&line).unwrap())
}

// Note: Some tags even with a post count greater than zero
// may have been removed.
fn parse_tags() -> Tags {
    read_all_json_lines(&format!("{DANBOORU_DATASET_ROOT}/tags*"))
        .flat_map(|tag: RawTag| {
            if tag.post_count == 0 {
                return None;
            }

            Some(Tag {
                id: TagId(tag.id),
                name: tag.name,
                category: tag.category,
                post_count: tag.post_count,
            })
        })
        .map(|tag| (tag.id, tag))
        .collect()
}

fn create_tag_names_to_ids(tags: &Tags) -> TagNamesToIds {
    tags.values()
        .map(|tag| (tag.name.clone(), tag.id))
        .collect()
}

fn parse_posts(tag_names_to_ids: &TagNamesToIds) -> Posts {
    read_all_json_lines(&format!("{DANBOORU_DATASET_ROOT}/post*"))
        .flat_map(|post: RawPost| {
            let id = match post.id {
                None => return None,
                Some(id) => PostId(id),
            };

            let tags = post
                .tag_string
                .split_ascii_whitespace()
                .filter_map(|tag| tag_names_to_ids.get(tag))
                .cloned()
                .collect();

            Some(Post {
                id,
                tags,
                score: post.score,
                rating: post.rating.chars().next().unwrap(),
                created_at: post.created_at,
            })
        })
        .map(|post| (post.id, post))
        .collect()
}

fn create_tag_posts(posts: &Posts) -> TagPosts {
    println!("[create] tags to posts");
    let mut tag_posts: TagPosts = FxHashMap::default();
    for post in posts.values().progress() {
        for &tag in &post.tags {
            tag_posts.entry(tag).or_default().push(post.id);
        }
    }
    tag_posts
}

fn rank_top_n_tags_for_tag(
    posts: &Posts,
    tag_posts: &TagPosts,
    tag: TagId,
    n: usize,
) -> Vec<(TagId, usize)> {
    let mut tag_counts: FxHashMap<TagId, usize> = FxHashMap::default();
    let post_ids_with_tag = &tag_posts[&tag];
    for post_id in post_ids_with_tag {
        for &tag_id in &posts[post_id].tags {
            if tag_id != tag {
                *tag_counts.entry(tag_id).or_default() += 1;
            }
        }
    }

    let mut tag_counts: Vec<_> = tag_counts.into_iter().collect();
    tag_counts.sort_by_key(|(id, count)| (Reverse(*count), *id));
    tag_counts.truncate(n);
    tag_counts
}

fn rank_top_n_tags_for_all_tags(
    posts: &Posts,
    tag_posts: &TagPosts,
    n: usize,
) -> FxHashMap<TagId, Vec<(TagId, usize)>> {
    println!("[create] rank tags for all tags");
    tag_posts
        .keys()
        .progress()
        .par_bridge()
        .map(|&tag_id| (tag_id, rank_top_n_tags_for_tag(posts, tag_posts, tag_id, n)))
        .collect()
}

fn average_post_score_for_tag(posts: &Posts, tag_posts: &TagPosts, tag: TagId) -> f64 {
    let mut cumulative_score = 0;
    let post_ids = &tag_posts[&tag];
    for post_id in post_ids {
        cumulative_score += posts[post_id].score;
    }
    cumulative_score as f64 / post_ids.len() as f64
}

fn average_post_score_for_all_tags(
    posts: &Posts,
    tag_posts: &TagPosts,
    minimum_post_count: usize,
) -> FxHashMap<TagId, f64> {
    println!("[create] average post score for all tags");
    tag_posts
        .iter()
        .progress()
        .par_bridge()
        .filter(|(_, post_ids)| post_ids.len() >= minimum_post_count)
        .map(|(&tag_id, _)| (tag_id, average_post_score_for_tag(posts, tag_posts, tag_id)))
        .collect()
}

fn write_to_json<T: Serialize>(path: &Path, data: &T) {
    println!("[write/json] {}", path.display());
    let writer = BufWriter::new(File::create(path).unwrap());
    serde_json::to_writer(writer, data).unwrap();
}

fn main() {
    let tags = parse_tags();
    write_to_json(Path::new("../processed/tags.json"), &tags);

    let tag_names_to_ids = create_tag_names_to_ids(&tags);
    write_to_json(
        Path::new("../processed/tag_names_to_ids.json"),
        &tag_names_to_ids,
    );

    let posts = parse_posts(&tag_names_to_ids);
    write_to_json(Path::new("../processed/posts.json"), &posts);

    let tag_posts = create_tag_posts(&posts);
    let tags_with_rankings = rank_top_n_tags_for_all_tags(&posts, &tag_posts, 100);
    write_to_json(
        Path::new("../processed/tags_with_rankings.json"),
        &tags_with_rankings,
    );

    let tags_with_average_score = average_post_score_for_all_tags(&posts, &tag_posts, 1000);
    write_to_json(
        Path::new("../processed/tags_with_average_score.json"),
        &tags_with_average_score,
    );
}
