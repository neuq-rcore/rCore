import json
import sys

TEST_SKIPED = 0
TEST_FAILED = 1
TEST_PASSED = 2

def get_prefix(status: int):
    if status == TEST_FAILED:
        # Red "Failed"
        return "\033[31mFailed\033[0m"
    elif status == TEST_SKIPED:
        # Bright Yellow "Half"
        return "\033[93mSkiped\033[0m"
    elif status == TEST_PASSED:
        # Green "Passed"
        return "\033[32mPassed\033[0m"

def visualize(data):
    total_tests = 0
    failed = 0
    skiped = 0
    passed = 0

    total_scores = 0
    score = 0
    for test in data:
        total_tests += 1

        name = test['name']
        num_total = test['all']
        num_passed = test['passed']
        judgements = len(test['results'])

        total_scores += num_total
        score += num_passed

        status = 0

        if judgements == 0:
            skiped += 1
            status = TEST_SKIPED
        elif num_passed == num_total:
            passed += 1
            status = TEST_PASSED
        else:
            failed += 1
            status = TEST_FAILED

        line = "  " + get_prefix(status) + " " + name + " [" + str(num_passed) + "/" + str(num_total) + "]"
        print(line)
    
    print()
    print("Total tests: " + str(total_tests))
    print("    Skiped: " + str(skiped))
    print("    Passed: " + str(passed))
    print("    Failed: " + str(failed))
    print()
    print("Scores: " + str(score) + "/" + str(total_scores))
    print()
    print("Raw test result and kernel output will be uploaded to the artifacts")

def read_output(file_name: str):
    with open(file_name, "r") as file:
        return file.read()

if __name__ == '__main__':
    file_name = sys.argv[1]
    output = read_output(file_name)
    data = json.loads(output)
    visualize(data)
