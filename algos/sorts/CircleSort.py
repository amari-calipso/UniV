class CircleSort:
    def converge(self, array, a, b):
        s = False
        while a <= b:
            if array[a] > array[b]:
                array[a].swap(array[b])
                s = True

            a += 1
            b -= 1

        return s

    def sorter(self, array, a, b):
        if b - a == 1:
            if array[a] > array[b]:
                array[a].swap(array[b])
                return True
            return False
        elif b - a < 1:
            return False
        
        s = self.converge(array, a, b)
        
        m = a + ((b - a) // 2)

        l = self.sorter(array, a,     m)
        r = self.sorter(array, m + 1, b)
        
        return s or l or r
    
    def sort(self, array, a, b):
        while self.sorter(array, a, b - 1):
            pass

@Sort(
    "Exchange Sorts",
    "Circle Sort",
    "Circle Sort"
)
def circleSortRun(array):
    CircleSort().sort(array, 0, len(array))