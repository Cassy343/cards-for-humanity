class Game extends React.Component {
    render() {
        return <div>
            <Board />
            <Hand />
            <div>
            {this.props.name}: {this.props.points}
            </div>
        </div>
    }
}