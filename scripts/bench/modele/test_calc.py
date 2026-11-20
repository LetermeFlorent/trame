from calc import maximum, mean


def test_calc():
    assert mean([2, 4, 6]) == 4
    assert mean([10]) == 10
    assert maximum([1, 9, 3]) == 9


test_calc()
print("ok")
