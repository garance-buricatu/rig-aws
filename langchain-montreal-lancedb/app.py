import asyncio
from langchain_community.vectorstores import LanceDB
from langchain_openai import ChatOpenAI, OpenAIEmbeddings
from langchain_core.output_parsers import StrOutputParser
from langchain_core.runnables import RunnablePassthrough
from langchain import hub


def lambda_handler(event, context):
    return asyncio.run(main())
    
async def main(event: dict):
    llm = ChatOpenAI(model="gpt-4o")

    vectorstore = LanceDB(
        uri="",
        embedding=OpenAIEmbeddings(model='text-embedding-ada-002')
    )

    retriever = vectorstore.as_retriever()
    prompt = hub.pull("rlm/rag-prompt")


    rag_chain = (
        {"context": retriever | format_docs, "question": RunnablePassthrough()}
        | prompt
        | llm
        | StrOutputParser()
    )

    return {"response": rag_chain.invoke(event["prompt"]) }

def format_docs(docs):
    return "\n\n".join(doc.page_content for doc in docs)