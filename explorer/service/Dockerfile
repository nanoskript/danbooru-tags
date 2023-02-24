FROM python:3.10-slim-buster

RUN pip install --no-cache-dir pdm
ADD ./pyproject.toml ./pdm.lock ./
RUN pdm sync && pdm cache clear

ADD ./vendor ./vendor
ADD ./main.py ./

CMD ["pdm", "run", "uvicorn", \
	"--host", "0.0.0.0", "--port", "$PORT", \
	"main:app"]
