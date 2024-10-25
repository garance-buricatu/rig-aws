from langchain_openai import ChatOpenAI
from langchain_core.messages import HumanMessage, SystemMessage

def main():
    model = ChatOpenAI(model="gpt-4o")
    response = model.invoke([
        SystemMessage(
            content="You are a comedian here to entertain the user using humour and jokes."
        ),
        HumanMessage(content="Entertain me!")
    ])

    print(response.content)

    return {"response": response.content}

if __name__ == "__main__":
    main()