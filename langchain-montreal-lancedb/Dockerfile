FROM public.ecr.aws/lambda/python:3.12-x86_64

WORKDIR ${LAMBDA_TASK_ROOT}

ARG FUNCTION_NAME

# Copy function code
COPY ${FUNCTION_NAME}.py ${LAMBDA_TASK_ROOT}
COPY ${FUNCTION_NAME}_requirements.txt ${LAMBDA_TASK_ROOT}
COPY __init__.py ${LAMBDA_TASK_ROOT}
COPY montreal.py ${LAMBDA_TASK_ROOT}

RUN set -eux; \
    REQUIREMENTS_FILE="${FUNCTION_NAME}_requirements.txt"; \
    pip install -r "${REQUIREMENTS_FILE}"

# Override with "loader.handler" when running loader image
CMD [ "app.handler" ]

# Local build command: docker build -t <AWS account id>.dkr.ecr.us-east-1.amazonaws.com/langchain/montreal-lancedb/<loader/app>:latest --build-arg FUNCTION_NAME=<loader/app> .