class BinaryDoubleInsertionSort:
    def insertToLeft(this, array, a, b, temp, idx):
        while a > b:
            array[a].write(array[a-1].noMark())
            a -= 1
        array[b].writeRestoreIdx(temp, idx)

    def insertToRight(this, array, a, b, temp, idx):
        while a < b:
            array[a].write(array[a+1].noMark())
            a += 1
        array[a].writeRestoreIdx(temp, idx)

    def sort(this, array, a, b):
        if b-a < 2:
            return
        j = a+(b-a-2)//2+1
        i = a+(b-a-1)//2
        if j > i and array[i] > array[j]:
            array[i].swap(array[j])
        i -= 1
        j += 1
        l: Value
        r: Value
        lIdx: int
        rIdx: int
        m: int
        while j < b:
            if array[i] > array[j]:
                l, lIdx = array[j].readNoMark()
                r, rIdx = array[i].readNoMark()
                m = lrBinarySearch(array, i+1, j, l, False)
                this.insertToRight(array, i, m-1, l, lIdx)
                this.insertToLeft(array, j, lrBinarySearch(
                    array, m, j, r, True), r, rIdx)
            else:
                l, lIdx = array[i].readNoMark()
                r, rIdx = array[j].readNoMark()
                m = lrBinarySearch(array, i+1, j, l, True)
                this.insertToRight(array, i, m-1, l, lIdx)
                this.insertToLeft(array, j, lrBinarySearch(
                    array, m, j, r, False), r, rIdx)
            i -= 1
            j += 1


@Sort("Insertion Sorts", "Binary Double Insertion Sort", "Bin. Double Insert")
def binaryDoubleInsertionSortRun(array):
    BinaryDoubleInsertionSort().sort(array, 0, len(array))
