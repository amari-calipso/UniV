class RotateMergeSort:
    def __init__(this, rot):
        if rot is None:
            this.rotate = UniV_getUserRotation("Select rotation algorithm (default: Gries-Mills)").indexed
        else:
            this.rotate = sortingVisualizer.getRotationByName(rot).indexed

    def rotateMerge(this, array, a, m, b):
        rotate = this.rotate
        m1: int
        m2: int
        m3: int
        if m-a >= b-m:
            m1 = a+(m-a)//2
            m2 = lrBinarySearch(array, m, b, array[m1], True)
            m3 = m1+m2-m
        else:
            m2 = m+(b-m)//2
            m1 = lrBinarySearch(array, a, m, array[m2], False)
            m3 = m2-m+m1
            m2 += 1
        rotate(array, m1, m, m2)
        if m2-m3-1 > 0 and b-m2 > 0:
            this.rotateMerge(array, m3+1, m2, b)
        if m1-a > 0 and m3-m1 > 0:
            this.rotateMerge(array, a, m1, m3)

    def rotateMergeParallel(this, array, a, m, b):
        rotate = this.rotate
        m1: int
        m2: int
        m3: int
        if m-a >= b-m:
            m1 = a+(m-a)//2
            m2 = lrBinarySearch(array, m, b, array[m1], True)
            m3 = m1+m2-m
        else:
            m2 = m+(b-m)//2
            m1 = lrBinarySearch(array, a, m, array[m2], False)
            m3 = m2-m+m1
            m2 += 1
        rotate(array, m1, m, m2)
        t0 = None
        t1 = None
        if m2-m3-1 > 0 and b-m2 > 0:
            t0 = Thread(
                this.rotateMergeParallel, [this, array, m3+1, m2, b])
            Thread_start(t0)
        if m1-a > 0 and m3-m1 > 0:
            t1 = Thread(
                this.rotateMergeParallel, [this, array, a, m1, m3])
            Thread_start(t1)
        if t0 is not None:
            Thread_join(t0)
        if t1 is not None:
            Thread_join(t1)

    def sort(this, array, a, b):
        l = b-a
        j = 1
        while j < l:
            i = a
            while i+2*j <= b:
                this.rotateMerge(array, i, i+j, i+2*j)
                i += 2*j
            if i+j < b:
                this.rotateMerge(array, i, i+j, b)
            j *= 2

    def sortParallel(this, array, a, b):
        if b-a > 1:
            m = a+((b-a)//2)
            t0 = Thread(this.sortParallel, [this, array, a, m])
            t1 = Thread(this.sortParallel, [this, array, m, b])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
            this.rotateMergeParallel(array, a, m, b)


@Sort("Merge Sorts", "Rotate Merge Sort", "Rotate Merge")
def rotateMergeSortRun(array):
    RotateMergeSort(None).sort(array, 0, len(array))


@Sort("Merge Sorts", "Rotate Merge Sort (Parallel)", "Rotate Merge (Parallel)")
def rotateMergeSortRun(array):
    RotateMergeSort(None).sortParallel(array, 0, len(array))