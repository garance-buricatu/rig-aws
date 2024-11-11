FROM public.ecr.aws/lambda/python:3.10-x86_64

WORKDIR ${LAMBDA_TASK_ROOT}

ARG FUNCTION_NAME

# Copy function code
COPY . ${LAMBDA_TASK_ROOT}

RUN set -eux; \
    REQUIREMENTS_FILE="${FUNCTION_NAME}_requirements.txt"; \
    pip install -r "${REQUIREMENTS_FILE}"

CMD [ "${FUNCTION_NAME}.handler" ]