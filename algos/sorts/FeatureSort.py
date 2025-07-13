class FeatureSort:
    def sortSubarray(this, subarray, mainArray):
        l = len(subarray)
        if l <= 1:
            return
        UtilsIterablesSort(len(mainArray), mainArray).sort(subarray, 0, l)

    def sort(this, array, a, b):
        min_: int
        max_: int
        min_, max_ = findMinMax(array, a, b)
        CONST = (b-a)/(max_-min_+1)
        aux = []
        for i in range(b - a + 1):
            List_invisiblePush(aux, Array(0))
        i = a
        while i < b:
            idx = int((array[i].readInt()-min_)*CONST)
            aux[idx].append(array[i])
            i += 1
        for i in range(b-a):
            this.sortSubarray(aux[i], array)
        i = 0
        r = a
        while i < len(aux):
            j = 0
            while j < len(aux[i]):
                array[r].write(aux[i][j])
                j += 1
                r += 1
            i += 1


@Sort("Distribution Sorts", "Feature Sort", "Feature Sort")
def featureSortRun(array):
    FeatureSort().sort(array, 0, len(array))
