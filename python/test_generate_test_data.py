#!/usr/bin/env python3

import pytest
import generate_test_data

def test_capital_case():
    assert generate_test_data.main(None) == 0