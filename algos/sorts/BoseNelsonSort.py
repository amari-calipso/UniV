class BoseNelsonSort:
    def __init__(this, end):
        this.end = end

    def compSwapCheck(this, array, a, b):
        if b < this.end:
            compSwap(array, a, b)

    def boseNelsonMerge(this, array, a1, a2, n, f):
        if f:
            i = 0
            while i < n:
                this.compSwapCheck(array, a1+i, a2+i)
                i += 1
        h = n//2
        if h > 1:
            this.boseNelsonMerge(array, a1, a2, h, False), this.boseNelsonMerge(
                array, a1+h, a2+h, h, False)
        if h > 0:
            this.boseNelsonMerge(array, a1+h, a2, h, True)

    def boseNelsonSort(this, array, a, n):
        h = n//2
        if h > 1:
            this.boseNelsonSort(
                array, a, h), this.boseNelsonSort(array, a+h, h)
        this.boseNelsonMerge(array, a, a+h, h, True)

    def mergeParallel(this, array, a1, l1, a2, l2):
        if l1 == 1 and l2 == 1:
            compSwap(array, a1, a2)
        elif l1 == 1 and l2 == 2:
            compSwap(array, a1, a2+1)
            compSwap(array, a1, a2)
        elif l1 == 2 and l2 == 1:
            compSwap(array, a1, a2)
            compSwap(array, a1+1, a2)
        else:
            m1 = l1//2
            m2 = (l2//2)if (l1 % 2 == 1)else ((l2+1)//2)
            t0 = Thread(this.mergeParallel, [this, array, a1, m1, a2, m2])
            t1 = Thread(this.mergeParallel, [this, array, a1+m1, l1-m1, a2+m2, l2-m2])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
            this.mergeParallel(array, a1+m1, l1-m1, a2, m2)

    def sortParallel(this, array, a, l):
        if l > 1:
            m = l//2
            t0 = Thread(this.sortParallel, [this, array, a, m])
            t1 = Thread(this.sortParallel, [this, array, a+m, l-m])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
            this.mergeParallel(array, a, m, a+m, l-m)


@Sort("Concurrent Sorts", "Bose Nelson Sort", "Bose Nelson")
def boseNelsonSortRun(array):
    BoseNelsonSort(len(array)).boseNelsonSort(
        array, 0, 2**math.ceil(math.log2(len(array))))


@Sort("Concurrent Sorts", "Bose Nelson Sort (Parallel)", "Bose Nelson (Parallel)")
def boseNelsonSortRun(array):
    BoseNelsonSort(len(array)).sortParallel(array, 0, len(array))
