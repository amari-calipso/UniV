import math


class WeaveSort:
    def __init__(this, end):
        this.end = end

    def compSwapCheck(this, array, a, b):
        if b < this.end:
            compSwap(array, a, b)

    def circleRec(this, array, p, n, g):
        h = n//2
        i = 0
        while i < h:
            this.compSwapCheck(array, p+i*g, p+(n-1-i)*g)
            i += 1
        if n >= 2:
            this.circleRec(array, p, h, g)
            this.circleRec(array, p+h*g, h, g)

    def weaveSort(this, array, p, n, g):
        if n >= 2:
            h = n//2
            this.weaveSort(array, p, h, 2*g)
            this.weaveSort(array, p+g, h, 2*g)
        this.circleRec(array, p, n, g)

    def circleRecParallel(this, array, p, n, g):
        h = n//2
        i = 0
        while i < h:
            this.compSwapCheck(array, p+i*g, p+(n-1-i)*g)
            i += 1
        if n >= 2:
            t0 = Thread(
                this.circleRecParallel, [this, array, p, h, g])
            t1 = Thread(
                this.circleRecParallel, [this, array, p+h*g, h, g])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)

    def weaveSortParallel(this, array, p, n, g):
        if n >= 2:
            h = n//2
            t0 = Thread(
                this.weaveSortParallel, [this, array, p, h, 2*g])
            t1 = Thread(
                this.weaveSortParallel, [this, array, p+g, h, 2*g])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
        this.circleRecParallel(array, p, n, g)


@Sort("Concurrent Sorts", "Weave Sorting Network", "Weave")
def weaveSortRun(array):
    WeaveSort(len(array)).weaveSort(
        array, 0, 2**math.ceil(math.log2(len(array))), 1)


@Sort("Concurrent Sorts", "Weave Sorting Network (Parallel)", "Weave (Parallel)")
def weaveSortRun(array):
    WeaveSort(len(array)).weaveSortParallel(array, 0, 2**math.ceil(math.log2(len(array))), 1)
