from setuptools import setup, find_packages

setup(
    name="aegis-gateway-sdk",
    version="0.1.0",
    description="Python SDK for AEGIS Gateway API",
    packages=find_packages(),
    install_requires=[
        "httpx>=0.27.0",
    ],
    python_requires=">=3.9",
)
