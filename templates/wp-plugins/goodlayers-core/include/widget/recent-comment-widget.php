<?php
	/**
	 * A widget that show recent comment ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_recent_comment_widget');
	if( !function_exists('gdlr_core_recent_comment_widget') ){
		function gdlr_core_recent_comment_widget() {
			register_widget( 'Goodlayers_Core_Recent_Comment_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Recent_Comment_Widget') ){
		class Goodlayers_Core_Recent_Comment_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-recent-comment-widget', 
					esc_html__('Recent Comment Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('A widget that show latest comments', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$num_fetch = empty($instance['num-fetch'])? '3': $instance['num-fetch'];
				$num_excerpt = empty($instance['num-excerpt'])? '15': $instance['num-excerpt'];
					
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				// Widget Content
				$comments = get_comments(array(
					'number' => $num_fetch,
					'status' => 'approve',
					'post_status' => 'publish'
				));

				if( !empty($comments) ){
					echo '<div class="gdlr-core-recent-comment-widget">';
					foreach( $comments as $comment ){
						$comment_link = get_comment_link($comment);

						echo '<div class="gdlr-core-recent-comment-widget-item">';
						echo '<div class="gdlr-core-recent-comment-widget-avatar gdlr-core-media-image" >' . get_avatar($comment->user_id, 70) . '</div>';
						echo '<div class="gdlr-core-recent-comment-widget-content">';
						echo '<div class="gdlr-core-recent-comment-widget-author gdlr-core-title-font">' . get_comment_author_link($comment) . '</div>';
						echo '<div class="gdlr-core-recent-comment-widget-excerpt">';
						echo wp_trim_words($comment->comment_content, $num_excerpt, '...');
						echo '</div>';
						echo '</div>';
						echo '</div>';
					}
					echo '</div>';
				}
				
				// Closing of widget
				echo gdlr_core_escape_content($args['after_widget']);

			}

			// Widget Form
			function form( $instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					gdlr_core_widget_util::get_option(array(
						'title' => array(
							'type' => 'text',
							'id' => $this->get_field_id('title'),
							'name' => $this->get_field_name('title'),
							'title' => esc_html__('Title', 'goodlayers-core'),
							'value' => (isset($instance['title'])? $instance['title']: '')
						),
						'num-fetch' => array(
							'type' => 'text',
							'id' => $this->get_field_id('num-fetch'),
							'name' => $this->get_field_name('num-fetch'),
							'title' => esc_html__('Display Number', 'goodlayers-core'), 
							'value' => (isset($instance['num-fetch'])? $instance['num-fetch']: '3')
						),
						'num-excerpt' => array(
							'type' => 'text',
							'id' => $this->get_field_id('num-excerpt'),
							'name' => $this->get_field_name('num-excerpt'),
							'title' => esc_html__('Excerpt Number', 'goodlayers-core'), 
							'value' => (isset($instance['num-excerpt'])? $instance['num-excerpt']: '15')
						),
					));
				}

			}
			
			// Update the widget
			function update( $new_instance, $old_instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					return gdlr_core_widget_util::get_option_update($new_instance);
				}

				return $new_instance;
			}	
		} // class
	} // class_exists
?>