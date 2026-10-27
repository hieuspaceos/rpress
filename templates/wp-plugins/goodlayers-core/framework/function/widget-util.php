<?php
	/*	
	*	Goodlayers Widget Utility
	*/

	// include the shortcode support for the text widget
	add_filter('widget_text', 'do_shortcode');
	add_filter('widget_title', 'do_shortcode');

	if( !class_exists('gdlr_core_widget_util') ){
		class gdlr_core_widget_util{

			// get option as html
			static function get_option($options){

				$float = false;

				if( !empty($options) ){
					foreach($options as $option_slug => $option){

						echo '<p ';
						if( !empty($option['style']) ){
							$css_atts = array();
							if( in_array('half', $option['style']) ){
								$float = true;
								$css_atts += array('width'=>'48%', 'margin-right'=>'2%', 'float'=>'left');
							}
							if( in_array('margin-bottom-0', $option['style']) ){
								$css_atts += array('margin-bottom'=>'0px', 'margin-top'=>'0px');
							}
							echo gdlr_core_esc_style($css_atts);
						}
						echo '>';
						if( !empty($option['title']) ){
							echo '<label for="' . esc_attr($option['id']) . '" >' . gdlr_core_escape_content($option['title']) . '</label>';
						}

						switch( $option['type'] ){

							case 'text': 
								echo '<input type="text" class="widefat" id="' . esc_attr($option['id']) . '" name="' . esc_attr($option['name']) . '" ';
								echo 'value="' . (isset($option['value'])? esc_attr($option['value']): '') . '" />';
								break;

							case 'textarea': 
								echo '<textarea class="widefat" id="' . esc_attr($option['id']) . '" name="' . esc_attr($option['name']) . '" style="height: 150px;" >';
								echo esc_textarea($option['value']);
								echo '</textarea>';
								break;

							case 'colorpicker': 
								echo '<div class="clear" ></div>';
								echo '<input type="text" class="colorpicker" id="' . esc_attr($option['id']) . '" name="' . esc_attr($option['name']) . '" ';
								echo 'value="' . (isset($option['value'])? esc_attr($option['value']): '') . '" />';
?>
<script type="text/javascript">
	(function($){
		$(document).ready(function(){
			$('#<?php echo esc_js($option['id']); ?>').each(function(){
				var t = $(this);

				t.wpColorPicker({
					change: gdlr_core_debounce(function(event, ui){
						t.trigger('change');
					}, 500),
					clear: function(){
						t.trigger('change');
					}
				});
			});
		});
	})(jQuery);	
</script>
<?php
								break;

							case 'combobox':
								if( empty($option['value']) && !empty($option['default']) ){
									$option['value'] = $option['default'];
								}

								echo '<select class="widefat" id="' . esc_attr($option['id']) . '" name="' . esc_attr($option['name']) . '" >';
								foreach( $option['options'] as $key => $value ){
									echo '<option value="' . esc_attr($key) . '" ' . ((isset($option['value']) && $key == $option['value'])? 'selected': '') . ' >' . esc_html($value) . '</option>';
								}
								echo '</select>';
								break; 

							case 'multi-combobox':
								if( empty($option['value']) && !empty($option['default']) ){
									$option['value'] = $option['default'];
								}

								$values = empty($option['value'])? array(): explode(',', $option['value']);
								echo '<select multiple class="widefat" ';
								echo 'onChange="this.nextSibling.value = jQuery(this).val().join(',');" ';
								echo ' >'; 
								foreach( $option['options'] as $key => $value ){
									echo '<option value="' . esc_attr($key) . '" ' . (in_array($key, $values)? 'selected': '') . ' >' . esc_html($value) . '</option>';
								}
								echo '</select>';
								
								echo '<input type="hidden" id="' . esc_attr($option['id']) . '" name="' . esc_attr($option['name']) . '" value="' . esc_attr($option['value']) . '" />';
								break; 	

							case 'upload':
								$option['value'] = empty($option['value'])? '': $option['value'];
								$img_url = empty($option['value'])? '': wp_get_attachment_url($option['value']);

								echo '<br/>';
								echo '<img class="gdlr-core-widget-upload-thumbnail" src="' . esc_url($img_url) . '" alt="thumbnail" /><br/>';
								echo '<input class="gdlr-core-widget-upload-url" type="text" value="' . esc_url($img_url) . '" />';
								echo '<input class="gdlr-core-widget-upload-id" type="hidden" id="' . esc_attr($option['id']) . '" name="' . esc_attr($option['name']) . '" value="' . esc_attr($option['value']) . '" />';
								echo '<input class="gdlr-core-widget-upload-button" type="button" value="' . esc_html__('Upload', 'goodlayers-core') . '" />';
								break;

							default: break; 

						} // switch
						echo '</p>';

					} // $option['type']
				} // $options

				if( $float ){
					echo '<div style="clear: both; margin-bottom: 20px;" ></div>';
				} 
				
			}

			// option update
			static function get_option_update($instances){

				if( !empty($instances) ){
					foreach($instances as $key => $value){
						$instances[$key] = isset($value)? $value: '';
					}
				}

				return $instances;
			}

		} // class
	} // class_exists