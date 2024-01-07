import argparse

def main():
    # read in file from first arg
    parser = argparse.ArgumentParser(
        'convert_rmt.py',
        description='Convert Saleae Logic capture of RMT timings to a list of pulses'
    )
    parser.add_argument('input_file', help='Saleae Logic capture of RMT timings')
    args = parser.parse_args()

    with open(args.input_file, 'r') as f:
        lines = f.readlines()

    timings = []
    for line in lines[1:]:
        time, state = line.split(',')
        time = float(time) * 1000 * 1000
        timings.append((time, int(state)))

    last_state, last_time = timings[0]
    for (time, state) in timings[1:]:
        duration = int(round(time - last_time))
        mode = 'high' if last_state == 0 else 'low'
        print(f'&{mode}_pulse({duration})?,')
        last_state = state
        last_time = time

if __name__ == '__main__':
    main()
