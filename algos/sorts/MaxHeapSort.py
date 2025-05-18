class MaxHeapSort:
    def siftDown(this, array, root, dist, start):
        while root <= dist//2:
            leaf = 2*root
            if leaf < dist and array[start+leaf-1] < array[start+leaf]:
                leaf += 1
            if array[start+root-1] < array[start+leaf-1]:
                array[start+root-1].swap(array[start+leaf-1])
                root = leaf
            else:
                break

    def heapify(this, array, a, b):
        length = b-a
        i = length//2
        while i >= 1:
            this.siftDown(array, i, length, a)
            i -= 1

    def sort(this, array, a, b):
        this.heapify(array, a, b)
        i = b-a
        while i > 1:
            array[a].swap(array[a+i-1])
            this.siftDown(array, 1, i-1, a)
            i -= 1


@Sort("Tree Sorts", "Max Heap Sort", "Max Heap Sort")
def maxHeapSortRun(array):
    MaxHeapSort().sort(array, 0, len(array))
