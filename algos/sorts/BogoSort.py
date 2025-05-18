def bogoSort(array, a, b):
    while not checkSorted(array, a, b):
        shuffleRandom(array, a, b)


@Sort("Impractical Sorts", "Bogo Sort", "Bogo Sort")
def bogoSortRun(array):
    if len(array) > 10:
        sel: int
        sel = sortingVisualizer.getUserSelection(
            ["No, go back", "Yes"], "Bogo Sort will take a very long time to finish on an array length of "+str(len(array))+". Are you sure you want to continue?")
        if sel == 0:
            return
    bogoSort(array, 0, len(array))
