class Die {
  constructor (public name: string, public roll: string) {}

  get element () {
    const die = document.createElement('div');
    die.id = this.name;
    die.className = 'die';

    const label = document.createElement('label');

    const input = document.createElement('input');
    input.className = 'dice-input';
    input.type = 'text';
    input.value = this.roll;
    input.addEventListener('input', (e) => {
      this.roll = (e.target as HTMLInputElement).value;
    })
    label.appendChild(input);

    const display = document.createElement('p');
    display.className = 'dice-output';
    const displayText = document.createTextNode('');
    display.appendChild(displayText);

    const button = document.createElement('button');
    button.className = 'pure-button';
    button.textContent = 'Roll';
    button.addEventListener('click', () => {
      roll(this.roll, display);
    })
    label.appendChild(button);

    die.appendChild(label);
    die.appendChild(display);

    return die;
  }
}

class State {
  constructor (public dice: Die[]) {
    this.dice = dice;
  }

  // Does not check for duplicates
  push (die: Die) {
    this.dice.push(die);
  }

  remove (name: string) {
    for (let [i, die] of this.dice.entries()) {
      if (die.name === name) {
        this.dice.splice(i, 1);
        // We just removed an element from the array
        i--;
      }
    }
  }

  display (name: string) {
    for (const die of this.dice) {
      if (die.name === name) {
        const dice = document.querySelector('#dice');
        dice!.replaceChild(die.element, dice!.childNodes[0]);
      }
    }
  }
}

function roll (roll: string, display: HTMLParagraphElement) {
  fetch('/dice', {
    method: 'post',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ roll })
  })
    .then((res) => {
      if (res.status === 422) {
        return res.text();
      }

      if (res.status !== 200) {
        throw new Error(`HTTP ${res.status}`);
      }

      return res.json();
    })
    .then((payload) => {
      const text = document.createTextNode(payload.roll);
      display.replaceChild(text, display.childNodes[0]);
    })
    .catch(showErrorDialog);
}

const showErrorDialog = (e: Error) => {
  alert(`Something went wrong!\n\n${e}\n\n${e.stack}`);
}

const dice = [new Die('default', '3x 3d20 *4 +1 s2')];
const state = new State(dice);
state.display('default');
