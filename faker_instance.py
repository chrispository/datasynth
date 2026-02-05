"""
Shared Faker instance for consistent fake data generation across the project.

This module provides a single, seeded Faker instance to ensure:
1. Memory efficiency (only one instance)
2. Reproducible results when seeded (useful for testing)
3. Consistent locale/configuration across all modules
"""

import os
from faker import Faker

# Create a shared Faker instance with optional seed from environment
# Set FAKER_SEED environment variable for reproducible results
_seed = os.getenv("FAKER_SEED")
if _seed is not None:
    try:
        _seed = int(_seed)
    except ValueError:
        _seed = None

fake = Faker()
if _seed is not None:
    fake.seed_instance(_seed)


def reseed(seed_value=None):
    """
    Reseed the shared Faker instance.
    
    Args:
        seed_value: Integer seed for reproducible random data.
                   If None, uses current time (non-deterministic).
    """
    if seed_value is not None:
        fake.seed_instance(seed_value)
    else:
        fake.seed_instance()


def get_seed():
    """Get the current seed value if set, else None."""
    return _seed
