"""Deterministic differential cases. Add a Java method, then one row below."""

# Enable a feature only after Klover supports its execution semantics.
FEATURES = {
    'primitive_entries': True,
    'immediate_constants': True,
    'locals': True,
    'int_arithmetic': True,
    'long_arithmetic': True,
    'floating_point': True,
    'conversions': True,
    'branches': True,
    'loops': True,
    'switches': True,
    'constant_pool_loads': False,
    'invokestatic': False,
    'class_initialization': False,
    'objects': False,
    'arrays': False,
    'virtual_calls': False,
    'exceptions': False,
    'gc': False,
}


def case(name, owner, method, descriptor, args=(), *requires):
    return {"name": name, "class": owner, "method": method, "descriptor": descriptor,
            "args": [str(value) for value in args],
            "requires": ["primitive_entries", *requires]}


# Arguments: decimal integers, or hexadecimal IEEE bits for float/double.
# Columns: name, class, method, descriptor, arguments, required features.
CASES = [
    # constants
    case('constants/minus_one', 'ConstantOps', 'intMinusOne', '()I', (), 'immediate_constants'),
    case('constants/zero', 'ConstantOps', 'intZero', '()I', (), 'immediate_constants'),
    case('constants/five', 'ConstantOps', 'intFive', '()I', (), 'immediate_constants'),
    case('constants/bipush', 'ConstantOps', 'byteImmediate', '()I', (), 'immediate_constants'),
    case('constants/sipush', 'ConstantOps', 'shortImmediate', '()I', (), 'immediate_constants'),
    case('constants/long_one', 'ConstantOps', 'longOne', '()J', (), 'immediate_constants'),
    case('constants/double_one', 'ConstantOps', 'doubleOne', '()D', (), 'immediate_constants', 'floating_point'),
    # arithmetic
    case('arithmetic/local_add', 'EntryOps', 'test', '()I', (), 'int_arithmetic', 'locals', 'immediate_constants'),
    case('arithmetic/add_zero', 'ArithmeticOps', 'iadd', '(II)I', (0, 0), 'int_arithmetic', 'locals'),
    case('arithmetic/add_negative', 'ArithmeticOps', 'iadd', '(II)I', (-17, 5), 'int_arithmetic', 'locals'),
    case('arithmetic/add_overflow', 'ArithmeticOps', 'iadd', '(II)I', (2147483647, 1), 'int_arithmetic', 'locals'),
    case('arithmetic/sub_underflow', 'ArithmeticOps', 'isub', '(II)I', (-2147483648, 1), 'int_arithmetic', 'locals'),
    case('arithmetic/multiply_overflow', 'ArithmeticOps', 'imul', '(II)I', (65536, 65536), 'int_arithmetic', 'locals'),
    case('arithmetic/divide_negative', 'ArithmeticOps', 'idiv', '(II)I', (-7, 3), 'int_arithmetic', 'locals'),
    case('arithmetic/divide_min', 'ArithmeticOps', 'idiv', '(II)I', (-2147483648, -1), 'int_arithmetic', 'locals'),
    case('arithmetic/remainder_negative', 'ArithmeticOps', 'irem', '(II)I', (-7, 3), 'int_arithmetic', 'locals'),
    case('arithmetic/negate_min', 'ArithmeticOps', 'ineg', '(I)I', (-2147483648,), 'int_arithmetic', 'locals'),
    case('arithmetic/shift_masked', 'ArithmeticOps', 'ishl', '(II)I', (1, 32), 'int_arithmetic', 'locals'),
    case('arithmetic/unsigned_shift_negative_distance', 'ArithmeticOps', 'iushr', '(II)I', (-1, -1), 'int_arithmetic', 'locals'),
    case('arithmetic/signed_shift', 'ArithmeticOps', 'ishr', '(II)I', (-8, 2), 'int_arithmetic', 'locals'),
    case('arithmetic/xor', 'ArithmeticOps', 'ixor', '(II)I', (-1, 123), 'int_arithmetic', 'locals'),
    case('arithmetic/increment_overflow', 'ArithmeticOps', 'iinc', '(I)I', (2147483647,), 'int_arithmetic', 'locals'),
    case('arithmetic/long_add_overflow', 'ArithmeticOps', 'ladd', '(JJ)J', (9223372036854775807, 1), 'long_arithmetic', 'locals'),
    case('arithmetic/long_divide_min', 'ArithmeticOps', 'ldiv', '(JJ)J', (-9223372036854775808, -1), 'long_arithmetic', 'locals'),
    case('arithmetic/long_remainder_min', 'ArithmeticOps', 'lrem', '(JJ)J', (-9223372036854775808, -1), 'long_arithmetic', 'locals'),
    case('arithmetic/long_shift_masked', 'ArithmeticOps', 'lshl', '(JI)J', (1, 64), 'long_arithmetic', 'locals'),
    case('arithmetic/long_unsigned_shift', 'ArithmeticOps', 'lushr', '(JI)J', (-1, 63), 'long_arithmetic', 'locals'),
    case('arithmetic/float_add', 'ArithmeticOps', 'fadd', '(FF)F', ('3fc00000', '40100000'), 'floating_point', 'locals'),
    case('arithmetic/float_negative_zero', 'ArithmeticOps', 'fmul', '(FF)F', ('80000000', '3f800000'), 'floating_point', 'locals'),
    case('arithmetic/float_infinity', 'ArithmeticOps', 'fdiv', '(FF)F', ('3f800000', '00000000'), 'floating_point', 'locals'),
    case('arithmetic/float_nan', 'ArithmeticOps', 'fdiv', '(FF)F', ('00000000', '00000000'), 'floating_point', 'locals'),
    case('arithmetic/double_add', 'ArithmeticOps', 'dadd', '(DD)D', ('3ff8000000000000', '4002000000000000'), 'floating_point', 'locals'),
    case('arithmetic/double_negative_zero', 'ArithmeticOps', 'dneg', '(D)D', ('0000000000000000',), 'floating_point', 'locals'),
    case('arithmetic/double_nan', 'ArithmeticOps', 'ddiv', '(DD)D', ('0000000000000000', '0000000000000000'), 'floating_point', 'locals'),
    case('arithmetic/double_remainder', 'ArithmeticOps', 'drem', '(DD)D', ('c01e000000000000', '4000000000000000'), 'floating_point', 'locals'),
    # conversions
    case('conversions/int_float_rounding', 'Wide', 'i2f', '(I)F', (16777217,), 'conversions', 'locals', 'floating_point'),
    case('conversions/long_double_rounding', 'Wide', 'l2d', '(J)D', (9007199254740993,), 'conversions', 'locals', 'floating_point'),
    case('conversions/long_int_truncation', 'Wide', 'l2i', '(J)I', (8589934591,), 'conversions', 'locals', 'floating_point'),
    case('conversions/float_nan_to_int', 'Wide', 'f2i', '(F)I', ('7fc00001',), 'conversions', 'locals', 'floating_point'),
    case('conversions/double_infinity_to_long', 'Wide', 'd2l', '(D)J', ('7ff0000000000000',), 'conversions', 'locals', 'floating_point'),
    case('conversions/double_negative_zero_to_float', 'Wide', 'd2f', '(D)F', ('8000000000000000',), 'conversions', 'locals', 'floating_point'),
    case('conversions/byte', 'InstructionOps', 'narrowByte', '(I)I', (255,), 'conversions', 'locals'),
    case('conversions/char', 'InstructionOps', 'narrowChar', '(I)I', (-1,), 'conversions', 'locals'),
    case('conversions/short', 'InstructionOps', 'narrowShort', '(I)I', (65535,), 'conversions', 'locals'),
    # branches
    case('branches/equal_boundary', 'ControlFlow', 'intsEqual', '(II)Z', (2147483647, 2147483647), 'branches', 'locals', 'immediate_constants'),
    case('branches/less_extremes', 'ControlFlow', 'intLessThan', '(II)Z', (-2147483648, 2147483647), 'branches', 'locals', 'immediate_constants'),
    case('branches/greater_equal_boundary', 'ControlFlow', 'intGreaterOrEqual', '(II)Z', (0, 0), 'branches', 'locals', 'immediate_constants'),
    case('branches/negative_zero_boundary', 'ControlFlow', 'isNegative', '(I)Z', (0,), 'branches', 'locals', 'immediate_constants'),
    case('branches/negative_true', 'ControlFlow', 'isNegative', '(I)Z', (-1,), 'branches', 'locals', 'immediate_constants'),
    case('branches/long_extremes', 'InstructionOps', 'longCompare', '(JJ)I', (-9223372036854775808, 9223372036854775807), 'branches', 'long_arithmetic', 'locals'),
    case('branches/float_nan', 'Wide', 'fbranch', '(FF)I', ('7fc00001', '3f800000'), 'branches', 'floating_point', 'locals'),
    # loops
    case('loops/zero_iterations', 'ControlFlow', 'sum', '(I)I', (0,), 'loops', 'branches', 'int_arithmetic', 'locals', 'immediate_constants'),
    case('loops/many_iterations', 'ControlFlow', 'sum', '(I)I', (10,), 'loops', 'branches', 'int_arithmetic', 'locals', 'immediate_constants'),
    case('loops/negative_bound', 'ControlFlow', 'countTo', '(I)I', (-1,), 'loops', 'branches', 'int_arithmetic', 'locals', 'immediate_constants'),
    case('loops/nested', 'AlgorithmOps', 'nestedSum', '(II)I', (4, 5), 'loops', 'branches', 'int_arithmetic', 'locals', 'immediate_constants'),
    # branches
    case('branches/table_first', 'SwitchOps', 'dense', '(I)I', (-1,), 'switches', 'locals', 'immediate_constants', 'int_arithmetic'),
    case('branches/table_last', 'SwitchOps', 'dense', '(I)I', (2,), 'switches', 'locals', 'immediate_constants', 'int_arithmetic'),
    case('branches/table_default', 'SwitchOps', 'dense', '(I)I', (3,), 'switches', 'locals', 'immediate_constants', 'int_arithmetic'),
    case('branches/lookup_case', 'SwitchOps', 'sparse', '(I)I', (1000,), 'switches', 'locals', 'immediate_constants', 'int_arithmetic'),
    case('branches/lookup_default', 'SwitchOps', 'sparse', '(I)I', (999,), 'switches', 'locals', 'immediate_constants', 'int_arithmetic'),
    case('branches/fallthrough', 'SwitchOps', 'denseFallThrough', '(I)I', (0,), 'switches', 'locals', 'immediate_constants', 'int_arithmetic'),
    # entries
    case('entries/byte', 'EntryOps', 'byteIdentity', '(B)B', (-128,), 'locals'),
    case('entries/char', 'EntryOps', 'charIdentity', '(C)C', (65535,), 'locals'),
    case('entries/short', 'EntryOps', 'shortIdentity', '(S)S', (-32768,), 'locals'),
    case('entries/boolean', 'EntryOps', 'booleanIdentity', '(Z)Z', (1,), 'locals'),
    case('entries/mixed_slots', 'EntryOps', 'mixed', '(IJFD)D', (3, 7, '3fc00000', '4002000000000000'), 'locals', 'conversions', 'floating_point', 'long_arithmetic'),
    case('entries/void', 'ControlFlow', 'doNothing', '()V', ()),
    # static_calls
    case('static_calls/direct_entry', 'StaticCallee', 'twice', '(I)I', (-5,), 'int_arithmetic', 'locals', 'immediate_constants'),
    case('static_calls/nested', 'DeferredCalls', 'nested', '(I)I', (7,), 'invokestatic', 'int_arithmetic', 'locals'),
    case('static_calls/cross_class', 'DeferredCalls', 'crossClass', '(I)I', (7,), 'invokestatic', 'int_arithmetic', 'locals'),
    # constants
    case('constants/pool_int', 'ConstantOps', 'intPoolConstant', '()I', (), 'constant_pool_loads'),
    case('constants/pool_double', 'ConstantOps', 'doublePoolConstant', '()D', (), 'constant_pool_loads', 'floating_point'),
    # exceptions
    case('exceptions/divide_by_zero', 'ArithmeticOps', 'idiv', '(II)I', (1, 0), 'exceptions', 'int_arithmetic', 'locals'),
]
