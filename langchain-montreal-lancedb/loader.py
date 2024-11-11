import os
import json
import asyncio
from langchain_community.document_loaders import JSONLoader
from langchain_community.vectorstores import LanceDB
from langchain_openai import OpenAIEmbeddings
from montreal import MontrealOpenDataClient, CategoryMetadata

def lambda_handler(event, context):
    return asyncio.run(main())
    
async def main():
    client = MontrealOpenDataClient()
    items = await client.search_all()

    data = [CategoryMetadata(item).to_paragraph() for item in items]

    file_path = "./open_data_results.json"

    with open(file_path, "w") as file:
        json.dump(data, file, indent=2)

    loader = JSONLoader(
        file_path='./open_data_results.json',
        jq_schema='.[]'
    )
    documents = loader.load()

    os.remove(file_path)

    # Note: dataset is too small to have a meaningful index (less than 5000 vectors)
    LanceDB.from_documents(documents, embeddings=OpenAIEmbeddings(model='text-embedding-ada-002'), uri="/mnt/efs")