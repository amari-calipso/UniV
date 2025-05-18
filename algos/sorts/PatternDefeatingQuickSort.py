class PDQSort:
    insertSortThreshold = 24
    nintherThreshold = 128
    partialInsertSortLimit = 8

    def log(this, n):
        n >>= 1
        log = 0
        while n != 0:
            log += 1
            n >>= 1
        return log

    def insertSort(this, array, begin, end):
        if begin == end:
            return
        i = begin+1
        while i < end:
            key: Value
            idx: int
            if array[i] < array[i-1]:
                key, idx = array[i].readNoMark()
                j = i-1
                while array[j] > key and j >= begin:
                    array[j+1].write(array[j])
                    j -= 1
                array[j+1].writeRestoreIdx(key, idx)
            i += 1

    def unguardInsertSort(this, array, begin, end):
        if begin == end:
            return
        i = begin+1
        while i < end:
            key: Value
            idx: int
            if array[i] < array[i-1]:
                key, idx = array[i].readNoMark()
                j = i-1
                while array[j] > key:
                    array[j+1].write(array[j])
                    j -= 1
                array[j+1].writeRestoreIdx(key, idx)
            i += 1

    def partialInsertSort(this, array, begin, end):
        if begin == end:
            return True
        i = begin+1
        limit = 0
        while i < end:
            if limit > this.partialInsertSortLimit:
                return False
            key: Value
            j = i-1
            idx: int
            if array[i] < array[i-1]:
                key, idx = array[i].readNoMark()
                while array[j] > key and j >= begin:
                    array[j+1].write(array[j])
                    j -= 1
                array[j+1].writeRestoreIdx(key, idx)
                limit += i-j+1
            i += 1
        return True

    def sortTwo(this, array, a, b):
        if array[b] < array[a]:
            array[a].swap(array[b])

    def sortThree(this, array, a, b, c):
        this.sortTwo(array, a, b)
        this.sortTwo(array, b, c)
        this.sortTwo(array, a, b)

    def partRight(this, array, begin, end):
        pivot = array[begin].copy()
        first = begin
        last = end
        while True:
            first += 1
            if not (array[first] < pivot):
                break
        if first-1 == begin:
            while True:
                if not (first < last):
                    break
                last -= 1
                if not (not array[last] < pivot):
                    break
        else:
            while True:
                last -= 1
                if not (not array[last] < pivot):
                    break
        alreadyParted = first >= last
        while first < last:
            array[first].swap(array[last])
            first += 1
            while array[first] < pivot:
                first += 1
            last -= 1
            while not array[last] < pivot:
                last -= 1
        pivotPos = first-1
        array[begin].write(array[pivotPos])
        array[pivotPos].write(pivot)
        return pivotPos, alreadyParted

    def partLeft(this, array, begin, end):
        pivot = array[begin].copy()
        first = begin
        last = end
        while True:
            last -= 1
            if not (pivot < array[last]):
                break
        if last+1 == end:
            while True:
                if not (first < last):
                    break
                first += 1
                if not (not pivot < array[first]):
                    break
        else:
            while True:
                first += 1
                if not (not pivot < array[first]):
                    break
        while first < last:
            array[first].swap(array[last])
            last -= 1
            while pivot < array[last]:
                last -= 1
            first += 1
            while not pivot < array[first]:
                first += 1
        pivotPos = last
        array[begin].write(array[pivotPos])
        array[pivotPos].write(pivot)
        return pivotPos

    def loop(this, array, begin, end, badAllowed):
        leftmost = True
        while True:
            size = end-begin
            if size < this.insertSortThreshold:
                if leftmost:
                    this.insertSort(array, begin, end)
                else:
                    this.unguardInsertSort(array, begin, end)
                return
            halfSize = size//2
            if size > this.nintherThreshold:
                this.sortThree(array, begin, begin+halfSize, end-1)
                this.sortThree(array, begin+1, begin+(halfSize-1), end-2)
                this.sortThree(array, begin+2, begin+(halfSize+1), end-3)
                this.sortThree(array, begin+(halfSize-1),
                               begin+halfSize, begin+(halfSize+1))
                array[begin].swap(array[begin+halfSize])
            else:
                this.sortThree(array, begin, begin+halfSize, end-1)
            if (not leftmost) and (not array[begin-1] < array[begin]):
                begin = this.partLeft(array, begin, end)+1
                continue
            pivotPos: int
            alreadyParted: bool
            pivotPos, alreadyParted = this.partRight(array, begin, end)
            leftSize = pivotPos-begin
            rightSize = end-(pivotPos+1)
            highUnbalance = leftSize < size/8 or rightSize < size/8
            if highUnbalance:
                badAllowed -= 1
                if badAllowed == 0:
                    MaxHeapSort().sort(array, begin, end)
                    return
                if leftSize >= this.insertSortThreshold:
                    sortingVisualizer.delay(1040)
                    array[begin].swap(array[begin+leftSize//4])
                    sortingVisualizer.delay(1040)
                    array[pivotPos-1].swap(array[pivotPos-leftSize//4])
                    if leftSize > this.nintherThreshold:
                        sortingVisualizer.delay(1040)
                        array[begin+1].swap(array[begin+(leftSize//4+1)])
                        sortingVisualizer.delay(1040)
                        array[begin+2].swap(array[begin+(leftSize//4+2)])
                        sortingVisualizer.delay(1040)
                        array[pivotPos-2].swap(array[pivotPos-(leftSize//4+1)])
                        sortingVisualizer.delay(1040)
                        array[pivotPos-3].swap(array[pivotPos-(leftSize//4+2)])
                if rightSize >= this.insertSortThreshold:
                    sortingVisualizer.delay(1040)
                    array[pivotPos+1].swap(array[pivotPos+(1+rightSize//4)])
                    sortingVisualizer.delay(1040)
                    array[end-1].swap(array[end-rightSize//4])
                    if rightSize > this.nintherThreshold:
                        sortingVisualizer.delay(1040)
                        array[pivotPos +
                              2].swap(array[pivotPos+(2+rightSize//4)])
                        sortingVisualizer.delay(1040)
                        array[pivotPos +
                              3].swap(array[pivotPos+(3+rightSize//4)])
                        sortingVisualizer.delay(1040)
                        array[end-2].swap(array[end-(1+rightSize//4)])
                        sortingVisualizer.delay(1040)
                        array[end-3].swap(array[end-(2+rightSize//4)])
            else:
                if alreadyParted and this.partialInsertSort(array, begin, pivotPos) and this.partialInsertSort(array, pivotPos+1, end):
                    return
            this.loop(array, begin, pivotPos, badAllowed)
            begin = pivotPos+1
            leftmost = False


@Sort("Quick Sorts", "Pattern-Defeating QuickSort", "PDQ Sort")
def pdqSortRun(array):
    pdq = PDQSort()
    pdq.loop(array, 0, len(array), pdq.log(len(array)))
