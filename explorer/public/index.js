import {h, render} from "https://cdn.skypack.dev/preact@10.11.2?min";
import {useEffect, useState, useRef} from "https://cdn.skypack.dev/preact@10.11.2/hooks?min";
import htm from "https://cdn.skypack.dev/htm@3.1.1?min";

import "https://cdnjs.cloudflare.com/ajax/libs/dayjs/1.11.8/dayjs.min.js";
import "https://cdnjs.cloudflare.com/ajax/libs/Chart.js/4.3.0/chart.umd.min.js";
import "https://cdn.jsdelivr.net/npm/chartjs-adapter-dayjs@1.0.0/+esm";

// Initialize htm with Preact.
const html = htm.bind(h);

// Global variables.
const API = "https://danbooru-tags-explorer.nanoskript.dev";

// Tags data.
const TAG_CATEGORIES = {
    0: {
        name: "General",
        color: "#009be6",
    },
    1: {
        name: "Artist",
        color: "#ff8a8b",
    },
    3: {
        name: "Copyright",
        color: "#c797ff",
    },
    4: {
        name: "Character",
        color: "#35c64a"
    },
    5: {
        name: "Meta",
        color: "#ead084",
    },
};

const TagSearchForm = ({query, updateQuery}) => {
    const [string, setString] = useState(query.get("tag") || "");
    const [completions, setCompletions] = useState([]);

    useEffect(() => {
        const controller = new AbortController();
        (async () => {
            try {
                const request = {signal: controller.signal};
                const response = await fetch(`${API}/tag_complete?prefix=${string}`, request);
                setCompletions(await response.json());
            } catch (_error) {
            }
        })();
        return () => controller.abort();
    }, [string]);

    const onSelect = (newString) => {
        setString(newString);
        updateQuery("tag", newString);
    };

    return html`
        <form onsubmit=${(e) => {
            e.preventDefault();
            onSelect(string);
        }}>
            <div class="tag-search-form">
                <div class="tag-search-input-container">
                    <input type="text" value=${string} class="tag-search-input"
                           placeholder="Tag name" autocomplete="off"
                           autocapitalize="none" spellcheck="false"
                           oninput=${(e) => setString(e.target.value)}/>
                    <dl class="tag-search-menu">
                        ${completions.map(({tag, category}) => html`
                            <dt
                                class="tag-search-menu-item"
                                onclick=${() => onSelect(tag)}
                            >
                                <span style=${`color: ${TAG_CATEGORIES[category].color};`}>${tag}</span>
                                <span style="font-size: smaller;"> — ${TAG_CATEGORIES[category].name}</span>
                            </dt>
                        `)}
                    </dl>
                </div>
                <input type="submit" value="Submit"/>
            </div>
        </form>
    `;
};

const TagPostsOverTime = ({query}) => {
    const string = (query.get("tag") || "").trim();
    if (string.length === 0) return html``;

    const [results, setResults] = useState(null);
    useEffect(() => (async () => {
        const response = await fetch(`${API}/tag_posts_over_time?tag=${string}`);
        setResults(response.ok ? await response.json() : null);
    })(), [query]);
    if (!results) return html``;

    const element = useRef(null);
    useEffect(() => {
        const xAxis = [];
        const yAxis = [];
        for (const [timestamp, count] of results) {
            xAxis.push(Date.parse(timestamp));
            yAxis.push(count);
        }

        const chart = new Chart(element.current, {
            type: "bar",
            data: {
                parsing: false,
                labels: xAxis,
                datasets: [{
                    data: yAxis,
                    backgroundColor: "royalblue",
                    barPercentage: 1,
                    categoryPercentage: 1,
                }],
            },
            options: {
                animation: false,
                scales: {
                    xAxes: [{
                        type: "time",
                        time: {
                            minUnit: "month",
                            tooltipFormat: "MMM YYYY",
                        },
                    }],
                    yAxes: [{
                        ticks: {
                            precision: 0,
                            beginAtZero: true,
                        },
                    }],
                },
                legend: {
                    display: false,
                },
            },
        });

        return () => chart.destroy();
    }, [results]);

    return html`
        <div>
            <h2>Posts created over time</h2>
            <canvas ref=${element}></canvas>
        </div>
    `;
};

const TagCategoriesFilter = ({shownCategories, setShownCategories}) => {
    return html`
        <details>
            <summary>Filter tag categories</summary>
            ${Object.entries(TAG_CATEGORIES).map(([key, {name, color}]) => html`
                <label class="tag-category-selection">
                    <input
                        type="checkbox"
                        checked=${shownCategories[key]}
                        onchange=${(e) => {
                            const newShownCategories = structuredClone(shownCategories);
                            newShownCategories[key] = e.target.checked;
                            setShownCategories(newShownCategories);
                        }}
                    />
                    <span style=${`color: ${color};`}>
                    ${" "}${name}
                    </span>
                </label>
            `)}
        </details>
    `;
};

const TagCorrelationsList = ({updateQuery, shownCategories, results}) => {
    return html`
        <dl style="margin-top: 0.5rem;">
            ${results.correlations
                .filter(({tag_category}) => shownCategories[tag_category])
                .map((correlation) => {
                    const category = TAG_CATEGORIES[correlation.tag_category];
                    return html`
                        <dt
                            class="related-tag-item"
                            onclick=${() => updateQuery("tag", correlation.tag)}
                        >
                            <span style=${`color: ${category.color};`}>${correlation.tag}</span>
                            <span style="font-size: smaller;"> — ${category.name}</span>
                            <span style="font-size: smaller;">
                            ${" "}· ${Math.round((correlation.n_correlated / results.n_posts_for_tag) * 100)}% 
                        </span>
                            <span style="font-size: smaller;" class="related-tag-item-hidden">
                            ${" "}(${correlation.n_correlated} / ${results.n_posts_for_tag})
                        </span>
                        </dt>
                    `;
                })}
        </dl>
    `;
};

const TagCorrelations = ({query, updateQuery}) => {
    const [shownCategories, setShownCategories] = useState(Object.fromEntries(
        Object.keys(TAG_CATEGORIES).map((key) => [key, true])
    ));

    const string = (query.get("tag") || "").trim();
    if (string.length === 0) return html``;

    const [results, setResults] = useState(null);
    useEffect(() => (async () => {
        const response = await fetch(`${API}/tag_correlations?tag=${string}`);
        setResults(response.ok ? await response.json() : null);
    })(), [query]);
    if (!results) return html``;

    return html`
        <div>
            <h2>Top related tags</h2>
            <${TagCategoriesFilter}
                shownCategories=${shownCategories}
                setShownCategories=${setShownCategories}
            />
            <${TagCorrelationsList}
                updateQuery=${updateQuery}
                shownCategories=${shownCategories}
                results=${results}
            />
        </div>
    `;
};

function usePageQuery() {
    const read = () => new URL(window.location).searchParams;
    const [query, setQuery] = useState(read());
    useEffect(() => {
        const listener = () => setQuery(read());
        window.addEventListener("popstate", listener);
        return () => window.removeEventListener("popstate", listener);
    }, []);
    return [query, setQuery];
}

const Page = () => {
    const [query, setQuery] = usePageQuery();
    const updateQuery = (key, value) => {
        const url = new URL(window.location);
        url.searchParams.set(key, value);
        window.history.pushState(null, "", url.toString());
        setQuery(url.searchParams);
    };

    return html`
        <${TagSearchForm}
            key=${query}
            query=${query}
            updateQuery=${updateQuery}
        />
        <${TagPostsOverTime}
            query=${query}
        />
        <${TagCorrelations}
            query=${query}
            updateQuery=${updateQuery}
        />
    `;
};

const main = document.querySelector("main");
main.innerHTML = "";
render(html`<${Page}/>`, main);