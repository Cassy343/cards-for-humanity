class Board extends React.Component {

    constructor(props) {
        super(props);

        this.state = {
            black_card: "",
            played_cards: [],
        }

        global_var.update_black_card = (black_card) => {
            this.setState({black_card: black_card})
        };

        global_var.update_played_cards = (played_cards) => {
            this.setState({played_cards: played_cards})
        };
    }

    render() {
        let played_cards;
        for(card of this.state.played_cards) {
            played_cards += <Card text={card.text} />;
        }

        return <div>
            {this.state.black_card && <Card text={this.state.black_card}/>}
            {played_cards}
        </div>;
    }
}