import random

@PivotSelection("First")
def firstPivot(array, a, b):
    return a


@PivotSelection("Last")
def lastPivot(array, a, b):
    return b-1


@PivotSelection("Middle")
def middlePivot(array, a, b):
    return a+((b-a)//2)


@PivotSelection("Median of three (unstable)")
def medianOfThreeUnstablePivot(array, a, b):
    medianOfThree(array, a, b)
    return a+((b-a)//2)


@PivotSelection("Median of three")
def medianOfThreePivot(array, a, b):
    return medianOfThreeIdx(array, a, a+(b-a)//2, b-1)


@PivotSelection("Random")
def randomPivot(array, a, b):
    return random.randint(a, b-1)
