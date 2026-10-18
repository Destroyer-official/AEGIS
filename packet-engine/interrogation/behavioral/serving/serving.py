#!/usr/bin/env python3
"""
FastAPI Serving for LLM Refusal-Vector Classifier
§6.4 — LLM Refusal Vector Analysis — Production Serving

Provides /refusal-vector endpoint with <50ms p99 latency target.
"""

import os
import time
import logging
from typing import List, Optional
from pathlib import Path

import numpy as np
import onnxruntime as ort
from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field
import uvicorn

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# ── Request/Response Models ──────────────────────────────────────────────────

class RefusalVectorRequest(BaseModel):
    """Request for refusal-vector classification."""
    responses: List[str] = Field(
        ...,
        min_length=8,
        max_length=8,
        description="Exactly 8 responses to the standardized refusal prompts"
    )
    model_id: Optional[str] = Field(
        None,
        description="Optional model identifier for tracking"
    )

class RefusalVectorResponse(BaseModel):
    """Response from refusal-vector classification."""
    model_family: str
    confidence: float
    similarity_scores: List[float]
    per_prompt_scores: List[float]
    processing_ms: float
    model_version: str = "1.0.0"

class HealthResponse(BaseModel):
    status: str
    model_loaded: bool
    model_version: str
    uptime_seconds: float


# ── Model Loader ─────────────────────────────────────────────────────────────

class RefusalVectorModel:
    """ONNX model wrapper for refusal-vector classification."""
    
    CLASSES = ["GPT-4", "Claude", "Llama-3", "Gemini", "Mistral"]
    FEATURE_DIMS = 20
    NUM_PROMPTS = 8
    
    # Feature extraction patterns (matching Rust implementation)
    FEATURE_PATTERNS = [
        ("cannot", "cannot"),
        ("unable", "unable"),
        ("not_able", "not able"),
        ("wont", "won't"),
        ("im_sorry", "i'm sorry"),
        ("i_apologize", "i apologize"),
        ("unfortunately", "unfortunately"),
        ("i_cant", "i can't"),
        ("instead", "instead"),
        ("alternative", "alternative"),
        ("you_could", "you could"),
        ("i_can_help", "i can help"),
        ("as_an_ai", "as an ai"),
        ("as_language_model", "as a language model"),
        ("im_claude", "i'm claude"),
        ("i_am_claude", "i am claude"),
        ("harmful", "harmful"),
        ("dangerous", "dangerous"),
        ("illegal", "illegal"),
        ("against", "against"),
    ]
    
    def __init__(self, model_path: str):
        self.model_path = model_path
        self.session = None
        self.load_model()
        
    def load_model(self):
        """Load ONNX model with optimized session options."""
        if not os.path.exists(self.model_path):
            raise FileNotFoundError(f"Model not found: {self.model_path}")
        
        # Optimize session for low latency
        sess_options = ort.SessionOptions()
        sess_options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        sess_options.intra_op_num_threads = 1
        sess_options.inter_op_num_threads = 1
        sess_options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        
        self.session = ort.InferenceSession(
            self.model_path,
            sess_options=sess_options,
            providers=['CPUExecutionProvider']
        )
        
        # Warm up
        dummy = np.random.randn(1, self.FEATURE_DIMS).astype(np.float32)
        self.session.run(None, {'features': dummy})
        
        logger.info(f"✅ Model loaded from {self.model_path}")
        
    def extract_features(self, response: str) -> np.ndarray:
        """Extract 20-dimensional feature vector from response text."""
        lower = response.lower()
        words = lower.split()
        total = len(words)
        
        if total == 0:
            return np.zeros(self.FEATURE_DIMS, dtype=np.float32)
        
        total_f = float(total)
        features = np.zeros(self.FEATURE_DIMS, dtype=np.float32)
        
        for i, (name, pattern) in enumerate(self.FEATURE_PATTERNS):
            count = lower.count(pattern)
            features[i] = count / total_f
            
        return features
    
    def classify(self, responses: List[str]) -> dict:
        """Classify 8 responses and return model family with scores."""
        start_time = time.perf_counter()
        
        # Extract features from each response
        feature_vectors = []
        for response in responses:
            features = self.extract_features(response)
            feature_vectors.append(features)
        
        # Average features across all prompts
        combined = np.mean(feature_vectors, axis=0).astype(np.float32)
        
        # Run inference
        input_data = combined.reshape(1, -1)
        outputs = self.session.run(None, {'features': input_data})
        similarities = outputs[0][0]  # [5]
        
        # Get prediction
        pred_idx = int(np.argmax(similarities))
        confidence = float(similarities[pred_idx])
        model_family = self.CLASSES[pred_idx]
        
        # Per-prompt similarities against winning class
        per_prompt = []
        for fv in feature_vectors:
            fv_norm = fv / (np.linalg.norm(fv) + 1e-8)
            ref_vec = self.get_reference_vector(pred_idx)
            ref_norm = ref_vec / (np.linalg.norm(ref_vec) + 1e-8)
            sim = float(np.dot(fv_norm, ref_norm))
            per_prompt.append(sim)
        
        processing_ms = (time.perf_counter() - start_time) * 1000
        
        return {
            "model_family": model_family,
            "confidence": confidence,
            "similarity_scores": similarities.tolist(),
            "per_prompt_scores": per_prompt,
            "processing_ms": processing_ms
        }
    
    def get_reference_vector(self, class_idx: int) -> np.ndarray:
        """Get reference vector for a class (matches Rust reference_vectors)."""
        refs = {
            0: [0.08, 0.06, 0.02, 0.04, 0.03, 0.01, 0.02, 0.05,
                0.04, 0.03, 0.02, 0.06, 0.00, 0.00, 0.00, 0.00,
                0.05, 0.03, 0.02, 0.04],  # GPT-4
            1: [0.04, 0.03, 0.01, 0.06, 0.01, 0.00, 0.01, 0.08,
                0.03, 0.02, 0.01, 0.04, 0.00, 0.00, 0.09, 0.07,
                0.06, 0.04, 0.03, 0.05],  # Claude
            2: [0.06, 0.04, 0.03, 0.02, 0.00, 0.00, 0.00, 0.03,
                0.02, 0.01, 0.01, 0.02, 0.07, 0.05, 0.00, 0.00,
                0.04, 0.02, 0.03, 0.03],  # Llama-3
            3: [0.05, 0.05, 0.03, 0.03, 0.02, 0.01, 0.03, 0.04,
                0.05, 0.04, 0.03, 0.03, 0.00, 0.00, 0.00, 0.00,
                0.05, 0.03, 0.04, 0.06],  # Gemini
            4: [0.07, 0.05, 0.02, 0.03, 0.01, 0.01, 0.01, 0.04,
                0.03, 0.02, 0.02, 0.03, 0.00, 0.00, 0.00, 0.00,
                0.05, 0.02, 0.02, 0.04],  # Mistral
        }
        return np.array(refs[class_idx], dtype=np.float32)


# ── FastAPI App ──────────────────────────────────────────────────────────────

app = FastAPI(
    title="AEGIS Refusal-Vector Classifier",
    description="LLM behavioral fingerprinting via refusal-vector analysis (§6.4)",
    version="1.0.0"
)

# Global model instance
model: Optional[RefusalVectorModel] = None
start_time = time.time()

MODEL_PATH = os.environ.get("REFUSAL_MODEL_PATH", "refusal_vector.onnx")


@app.on_event("startup")
async def startup_event():
    global model
    try:
        model = RefusalVectorModel(MODEL_PATH)
        logger.info("🚀 Refusal-vector classifier ready")
    except Exception as e:
        logger.error(f"❌ Failed to load model: {e}")
        raise


@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint."""
    return HealthResponse(
        status="healthy" if model else "degraded",
        model_loaded=model is not None,
        model_version="1.0.0",
        uptime_seconds=time.time() - start_time
    )


@app.post("/refusal-vector", response_model=RefusalVectorResponse)
async def classify_refusal_vector(request: RefusalVectorRequest):
    """
    Classify LLM model family from 8 refusal responses.
    
    Target: <50ms p99 latency
    """
    if model is None:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    if len(request.responses) != 8:
        raise HTTPException(
            status_code=400,
            detail=f"Exactly 8 responses required, got {len(request.responses)}"
        )
    
    try:
        result = model.classify(request.responses)
        
        return RefusalVectorResponse(
            model_family=result["model_family"],
            confidence=result["confidence"],
            similarity_scores=result["similarity_scores"],
            per_prompt_scores=result["per_prompt_scores"],
            processing_ms=result["processing_ms"]
        )
    except Exception as e:
        logger.error(f"Classification error: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/metrics")
async def metrics():
    """Prometheus-style metrics endpoint."""
    if model is None:
        return {"model_loaded": False}
    
    # Quick benchmark
    test_responses = [
        "I cannot help with that request."] * 8
    
    latencies = []
    for _ in range(100):
        start = time.perf_counter()
        model.classify(test_responses)
        latencies.append((time.perf_counter() - start) * 1000)
    
    latencies.sort()
    return {
        "model_loaded": True,
        "p50_ms": latencies[50],
        "p95_ms": latencies[95],
        "p99_ms": latencies[99],
        "mean_ms": sum(latencies) / len(latencies),
        "uptime_seconds": time.time() - start_time
    }


if __name__ == "__main__":
    port = int(os.environ.get("PORT", "8080"))
    uvicorn.run(
        "serving:app",
        host="0.0.0.0",
        port=port,
        workers=1,
        loop="asyncio",
        http="httptools",
        log_level="info"
    )