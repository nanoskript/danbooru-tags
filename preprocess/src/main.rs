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
    id: u32,
    name: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    category: u8,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    post_count: u32,
}

#[derive(Deserialize)]
struct RawPost {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    id: Option<u32>,
    tag_string: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    score: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    up_score: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    down_score: i32,
    rating: String,
    created_at: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct TagId(u32);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct PostId(u32);

#[derive(Serialize, Deserialize)]
struct Tag {
    id: TagId,
    name: String,
    category: u8,
    post_count: u32,
}

#[derive(Serialize, Deserialize)]
struct Post {
    id: PostId,
    tags: Vec<TagId>,
    /// It is not guaranteed that the score is equal to
    /// the difference between the up and down scores.
    score: i32,
    up_score: u32,
    /// Normalized to be positive.
    down_score: u32,
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
                up_score: post.up_score as u32,
                down_score: (-post.down_score) as u32,
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

fn truncate_timestamp_to_month(timestamp: &str) -> &str {
    let (index, _) = timestamp.match_indices("-").nth(1).unwrap();
    &timestamp[..index]
}

fn posts_added_over_time_for_tag<'a>(
    posts: &'a Posts,
    tag_posts: &TagPosts,
    tag: TagId,
) -> Vec<(&'a str, u32)> {
    let mut counts: FxHashMap<&str, u32> = FxHashMap::default();
    let post_ids = &tag_posts[&tag];
    for post_id in post_ids {
        let timestamp = &posts[post_id].created_at;
        let timestamp = truncate_timestamp_to_month(timestamp);
        *counts.entry(timestamp).or_default() += 1;
    }

    let mut data = Vec::from_iter(counts.into_iter());
    data.sort_unstable_by_key(|&(timestamp, _)| timestamp);
    data
}

fn posts_added_over_time_for_all_tags<'a>(
    posts: &'a Posts,
    tag_posts: &TagPosts,
) -> FxHashMap<TagId, Vec<(&'a str, u32)>> {
    println!("[create] posts added over time for all tags");
    tag_posts
        .iter()
        .progress()
        .par_bridge()
        .map(|(&tag_id, _)| {
            let counts = posts_added_over_time_for_tag(posts, tag_posts, tag_id);
            (tag_id, counts)
        })
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
    let tags_with_rankings = rank_top_n_tags_for_all_tags(&posts, &tag_posts, 50);
    write_to_json(
        Path::new("../processed/tags_with_rankings.json"),
        &tags_with_rankings,
    );

    let tags_with_average_score = average_post_score_for_all_tags(&posts, &tag_posts, 1000);
    write_to_json(
        Path::new("../processed/tags_with_average_score.json"),
        &tags_with_average_score,
    );

    let tags_with_posts_over_time = posts_added_over_time_for_all_tags(&posts, &tag_posts);
    write_to_json(
        Path::new("../processed/tags_with_posts_over_time.json"),
        &tags_with_posts_over_time,
    );
}
