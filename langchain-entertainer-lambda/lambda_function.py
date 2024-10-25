from langchain_openai import ChatOpenAI
from langchain_core.messages import HumanMessage, SystemMessage

def lambda_handler(event, context):
    model = ChatOpenAI(model="gpt-4o")
    response = model.invoke([
        SystemMessage(
            content="You are a comedian here to entertain the user using humour and jokes."
        ),
        HumanMessage(content=event["prompt"])
    ])

    print(response.content)

    return {"response": response.content}