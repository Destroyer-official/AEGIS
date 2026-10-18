"""Export 6Hit DQN model to ONNX for Triton Inference Server.

§3.6.1: Triton deployment for <10ms p99 latency, batch inference.
"""

from __future__ import annotations

import os

import torch

from dqn_model import ACTION_DIM, STATE_DIM, DuelingDQN


def export_onnx(
    checkpoint_path: str = "/tmp/aegis/checkpoints/dqn_global_final.pt",
    output_path: str = "triton_model_repository/6hit_dqn/1/model.onnx",
    opset_version: int = 17,
    allow_untrained: bool = False,
):
    """Export trained DQN to ONNX format.

    Fails closed if checkpoint is not found unless allow_untrained=True is explicitly set
    (e.g. for CI/local pipeline structure testing).
    """
    model = DuelingDQN(state_dim=STATE_DIM, action_dim=ACTION_DIM)

    permit_untrained = (
        allow_untrained
        or os.environ.get("AEGIS_ALLOW_UNTRAINED_MODEL", "false").lower() in ("1", "true")
    )

    if os.path.exists(checkpoint_path):
        checkpoint = torch.load(checkpoint_path, map_location="cpu", weights_only=True)
        # Handle both checkpoint formats
        state_dict = checkpoint.get("policy", checkpoint.get("model", checkpoint))
        model.load_state_dict(state_dict)
        print(f"Loaded trained checkpoint from {checkpoint_path}")
    elif permit_untrained:
        print(f"WARNING: Checkpoint not found at {checkpoint_path}, permitted untrained weights (allow_untrained=True)")
    else:
        raise FileNotFoundError(
            f"Production model checkpoint required but not found at '{checkpoint_path}'. "
            "Untrained models cannot be exported to production Triton repository. "
            "To permit untrained weights for structural testing, pass allow_untrained=True."
        )

    model.eval()

    # Dummy input for tracing
    dummy_input = torch.randn(1, STATE_DIM, dtype=torch.float32)

    # Export
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=opset_version,
        do_constant_folding=True,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={
            "input": {0: "batch_size"},
            "output": {0: "batch_size"},
        },
        verbose=False,
    )

    print(f"Exported ONNX model to {output_path}")

    # Verify export
    import onnx
    onnx_model = onnx.load(output_path)
    onnx.checker.check_model(onnx_model)
    print("ONNX model validation passed")

    # Test with onnxruntime
    import onnxruntime as ort
    ort_session = ort.InferenceSession(output_path)
    ort_inputs = {"input": dummy_input.numpy()}
    ort_outputs = ort_session.run(None, ort_inputs)
    print(f"ONNX Runtime test: input {ort_inputs['input'].shape} -> output {ort_outputs[0].shape}")

    # Compare with PyTorch
    with torch.no_grad():
        torch_output = model(dummy_input).numpy()

    max_diff = abs(torch_output - ort_outputs[0]).max()
    print(f"Max difference PyTorch vs ONNX: {max_diff:.6f}")
    assert max_diff < 1e-5, "ONNX output differs from PyTorch!"

    return output_path


def create_triton_config(
    model_name: str = "6hit_dqn",
    max_batch_size: int = 32,
    input_name: str = "input",
    output_name: str = "output",
) -> str:
    """Generate Triton model config.pbtxt."""
    config = f"""
name: "{model_name}"
platform: "onnxruntime_onnx"
max_batch_size: {max_batch_size}
input [
  {{
    name: "{input_name}"
    data_type: TYPE_FP32
    dims: [ {STATE_DIM} ]
  }}
]
output [
  {{
    name: "{output_name}"
    data_type: TYPE_FP32
    dims: [ {ACTION_DIM} ]
  }}
]
instance_group [
  {{
    count: 2
    kind: KIND_CPU
  }}
]
dynamic_batching {{
  preferred_batch_size: [ 8, 16, 32 ]
  max_queue_delay_microseconds: 100
}}
optimization {{
  execution_policy {{
    batcher: DYNAMIC_BATCHER
  }}
}}
"""
    return config


if __name__ == "__main__":
    # Export ONNX
    export_onnx()

    # Create Triton config
    config = create_triton_config()
    config_path = "triton_model_repository/6hit_dqn/config.pbtxt"
    os.makedirs(os.path.dirname(config_path), exist_ok=True)
    with open(config_path, "w") as f:
        f.write(config)
    print(f"Created Triton config at {config_path}")

    print("\n=== Triton Model Repository Structure ===")
    for root, dirs, files in os.walk("triton_model_repository"):
        level = root.replace("triton_model_repository", "").count(os.sep)
        indent = " " * 2 * level
        print(f"{indent}{os.path.basename(root)}/")
        for f in files:
            print(f"{indent}  {f}")