FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY controller.py .

RUN mkdir -p /var/lib/cost-controller/reports

ENV PYTHONUNBUFFERED=1

CMD ["python", "-u", "controller.py"]