FROM python:3.10-slim-buster

RUN pip install --no-cache-dir pdm
ADD ./pyproject.toml ./pdm.lock ./

# System dependencies.
RUN apt-get update && apt-get install -y curl g++
RUN pdm sync && pdm cache clear

# Vendor external assets.
RUN mkdir vendor
ADD ./vendor/download.sh ./vendor/
RUN (cd ./vendor && bash download.sh)

ADD ./main.py ./

CMD ["pdm", "run", "uvicorn", \
	"--host", "0.0.0.0", "--port", "$PORT", \
	"main:app"]
