class MergeSort:
    def __init__(this, length):
        this.aux = sortingVisualizer.createValueArray(length)

    def mergeParallel(this, array, a, m, b):
        left = a
        right = m
        aux = a
        while left < m and right < b:
            if array[left] <= array[right]:
                this.aux[aux].write(array[left])
                left += 1
            else:
                this.aux[aux].write(array[right])
                right += 1
            aux += 1
        while left < m:
            this.aux[aux].write(array[left])
            left += 1
            aux += 1
        while right < b:
            this.aux[aux].write(array[right])
            right += 1
            aux += 1
        aux = a
        while a < b:
            array[a].write(this.aux[aux])
            a += 1
            aux += 1

    def mergeSortParallel(this, array, a, b):
        if b-a > 1:
            m = a+((b-a)//2)
            t0 = Thread(
                this.mergeSortParallel, [this, array, a, m])
            t1 = Thread(
                this.mergeSortParallel, [this, array, m, b])
            Thread_start(t0)
            Thread_start(t1)
            Thread_join(t0)
            Thread_join(t1)
            this.mergeParallel(array, a, m, b)


@Sort("Merge Sorts", "Merge Sort (Parallel)", "Merge Sort (Parallel)")
def mergeSortParallelRun(array):
    MergeSort(len(array)).mergeSortParallel(array, 0, len(array))
