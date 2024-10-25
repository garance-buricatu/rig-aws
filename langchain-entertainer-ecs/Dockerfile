FROM public.ecr.aws/docker/library/python:bookworm

RUN apt-get update

WORKDIR /langchain-entertainer
COPY . /langchain-entertainer

RUN pip install -r requirements.txt

CMD [ "python", "./main.py"]
