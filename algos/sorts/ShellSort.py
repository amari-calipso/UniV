class ShellSort:
    seq = [8861, 3938, 1750, 701, 301, 132, 57, 23, 10, 4, 1]

    def sort(this, array, a, b):
        for gap in this.seq:
            if gap >= b-a:
                continue
            i = a+gap
            while i < b:
                tmp: Value
                idx: int
                tmp, idx = array[i].readNoMark()
                j = i
                while j >= a+gap and array[j-gap] > tmp:
                    array[j].write(array[j-gap].noMark())
                    j -= gap
                array[j].writeRestoreIdx(tmp, idx)
                i += 1

    def gappedInsert(this, array, a, b, gap):
        i = a+gap
        while i < b:
            tmp: Value
            idx: int
            tmp, idx = array[i].readNoMark()
            j = i
            while j >= a+gap and array[j-gap] > tmp:
                array[j].write(array[j-gap].noMark())
                j -= gap
            array[j].writeRestoreIdx(tmp, idx)
            i += gap

    def sortParallel(this, array, a, b):
        for gap in this.seq:
            if gap > b-a-1:
                continue
            threads = []
            for i in range(gap):
                List_invisiblePush(threads, Thread(this.gappedInsert, [this, array, a+i, b, gap]))
            for t in threads:
                Thread_start(t)
            for t in threads:
                Thread_join(t)


@Sort("Insertion Sorts", "Shell Sort", "Shell Sort")
def shellSortRun(array):
    ShellSort().sort(array, 0, len(array))


@Sort("Insertion Sorts", "Shell Sort (Parallel)", "Shell Sort (Parallel)")
def shellSortRun(array):
    ShellSort().sortParallel(array, 0, len(array))
