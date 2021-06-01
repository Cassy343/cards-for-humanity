let global_var = {};

class App extends React.Component {

    constructor(props) {
        super(props);
        this.state = {
            // State 0 = game select
            // State 1 = in game
            state: 1
        };
    }

    render() {
        if(this.props.state == "game_select") {
            return <ServerMenu />
        } else {
            return <Game name="test" points="0"/>
        }
    }
}

// export function update_black_card(black_card) {
//     global_var.update_black_card(black_card);
// }

// export function update_played_cards(played_cards) {
//     global_var.update_played_cards(played_cards);
// }

// export function update_hand(hand) {
//     global_var.update_hand(hand)
// }