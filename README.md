# danbooru-tags

## Reproduction of results

### System requirements

You should have a system:

- With at least 16 gigabytes (preferably 32 gigabytes) of memory for data processing
- That is running either macOS or Linux

### Required software

- Python 3.10 with the `pdm` package manager
- A nightly installation of the Rust toolchain and compiler

### Steps

1. Clone this repository:

   ```bash
   git clone https://github.com/nanoskript/danbooru-tags.git
   ```

2. Download the metadata dataset files from [Gwern's Danbooru2021 dataset](https://gwern.net/danbooru2021) server into
   the `datasets` folder. You can use `rsync` to do this:

   ```bash
   cd ./datasets
   rsync --recursive --verbose -P -z "rsync://176.9.41.242:873/danbooru2021/metadata/posts*" ./   
   rsync --recursive --verbose -P -z "rsync://176.9.41.242:873/danbooru2021/metadata/tags*" ./   
   ```

   The `-z` flag compresses the data as it's being transferred which is helpful if you have low network bandwidth.

   When you are done, these files should be present:

   ```bash
   $ ls -goL
   -rw-r--r--  1   812044658 Jan 22 22:18 posts000000000000.json
   -rw-r--r--  1   814218033 Jan 22 22:19 posts000000000001.json
   -rw-r--r--  1   815199391 Jan 22 22:21 posts000000000002.json
   -rw-r--r--  1   813704290 Jan 22 22:22 posts000000000003.json
   -rw-r--r--  1   816305499 Jan 22 22:23 posts000000000004.json
   -rw-r--r--  1   813301746 Jan 22 22:25 posts000000000005.json
   -rw-r--r--  1   816004693 Jan 22 22:26 posts000000000006.json
   -rw-r--r--  1   814542748 Jan 22 22:27 posts000000000007.json
   -rw-r--r--  1   815819036 Jan 22 22:29 posts000000000008.json
   -rw-r--r--  1   814014352 Jan 22 22:30 posts000000000009.json
   -rw-r--r--  1   814740526 Jan 22 22:31 posts000000000010.json
   -rw-r--r--  1   815458514 Jan 22 22:32 posts000000000011.json
   -rw-r--r--  1   167686206 Jan 22 22:39 tags000000000000.json
   ```

3. Preprocess the raw datasets:

   ```bash
   cd preprocess
   cargo +nightly run --release
   ```

   The resultant files will appear in the `processed` folder.

4. Install the dependencies for the analysis notebook:

   ```bash
   cd analysis
   pdm install
   ```

5. Open `main.ipynb` (in the `analysis` folder) and execute all cells. The graphs will render inside the notebook and
   will also be exported to `processed/plots` in the `plotly` library format.
