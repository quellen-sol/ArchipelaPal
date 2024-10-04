def clamp(min_val: int, max_val: int, value: int) -> int:
    """Clamp a value between a minimum and maximum."""
    return max(min_val, min(max_val, value))
